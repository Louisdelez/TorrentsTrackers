# Identity

Système d'identité cryptographique unifié. Implémenté dans `crates/identity`.

## Pourquoi

- L'utilisateur doit pouvoir s'identifier sur **toutes les sources et tous les chats** sans dépendre du système d'auth de chaque plateforme (compte GitHub ≠ compte Google ≠ ...).
- Les modérateurs doivent pouvoir **bannir une identité de manière portable** (la même clé qui spam sur 5 commus = banissable d'un coup).
- L'utilisateur doit pouvoir **migrer entre machines** en transportant sa clé.
- Les contributions doivent être **vérifiables** (pas d'usurpation possible).

## Choix techniques

| Aspect | Choix | Justification |
|---|---|---|
| Algorithme | Ed25519 | Standard moderne, signatures de 64 octets, vérification rapide, déjà standard dans Nostr / SSH / age |
| Encoding pubkey | bech32 (`npub1...`) | Lisible, copy-paste-friendly, compatible Nostr |
| Stockage clé privée | Keyring OS (libsecret / Keychain / Credential Manager) | Sécurisé par le système, pas en clair sur disque |
| Fallback | Fichier AES-256-GCM avec passphrase scrypt | Pour systèmes sans keyring (serveurs headless) |
| Hash | BLAKE3 | Rapide, moderne, parallélisable |
| Crate Rust | `ed25519-dalek` | Mature, audité, performant |

## Cycle de vie

### Au premier lancement

1. L'app détecte qu'il n'y a pas d'identité stockée.
2. Onboarding : explique au user le concept ("ta clé = ton identité, garde-la précieusement").
3. Génère une paire ed25519 avec `OsRng`.
4. Stocke la clé privée dans le keyring OS sous l'entrée `torrents-trackers/identity`.
5. Stocke la clé publique dans la table `identity` de SQLite (lecture rapide).
6. Propose à l'utilisateur d'**exporter immédiatement** un backup chiffré (fichier `.tt-identity` avec passphrase).

### À chaque démarrage

1. Lit la clé publique depuis SQLite.
2. Charge la clé privée depuis le keyring (lazy, seulement si une signature est nécessaire).

### Export / import

```sh
tt identity export ~/backup.tt-identity        # demande une passphrase
tt identity import ~/backup.tt-identity        # demande la passphrase
```

Format `.tt-identity` :

```
TT-IDENTITY-V1
<base64 nonce (12 bytes)>
<base64 ciphertext>
```

Ciphertext = `AES-256-GCM(key=scrypt(passphrase, salt=nonce[0..8]), plaintext=ed25519_seed_32_bytes)`.

## Signature d'une Entry

```rust
pub fn sign_entry(entry: &mut Entry, signer: &SigningKey) {
    let payload = canonical_payload(entry);    // sérialisation déterministe
    entry.signature = signer.sign(&payload);
    entry.contributor_pubkey = signer.verifying_key().into();
}

pub fn verify_entry(entry: &Entry) -> bool {
    let payload = canonical_payload(entry);
    let pk = ed25519_dalek::VerifyingKey::from_bytes(&entry.contributor_pubkey.0).unwrap();
    pk.verify_strict(&payload, &entry.signature).is_ok()
}
```

`canonical_payload` produit une représentation byte-stable de l'entry (champs concaténés avec longueurs préfixées). Permet la vérification déterministe entre clients.

## Bans

Une commu maintient une liste de pubkeys bannies dans son backend (un fichier `bans.json` dans le repo, par exemple). L'app la fetch en même temps que les entries et filtre côté client.

```rust
pub struct Ban {
    pub pubkey: PublicKey,
    pub reason: Option<String>,
    pub banned_at: DateTime<Utc>,
    pub banned_by: PublicKey,            // signature du modo
    pub signature: Signature,
}
```

L'utilisateur peut aussi maintenir une **blacklist personnelle** locale (filtre une pubkey sur toutes ses sources, indépendamment de la modération de chaque commu).

## Sécurité

- **Jamais loguer la clé privée**, même au niveau debug.
- **Zeroize** la clé en mémoire dès qu'elle n'est plus nécessaire (crate `zeroize`).
- **Vérifier toutes les signatures** au moment de l'insertion en base. Une entry avec signature invalide est rejetée silencieusement.
- **Refuser les entries non signées** par défaut (option `--accept-unsigned` pour les sources legacy).
- **Vérifier la fraîcheur** : `added_at` doit être dans une fenêtre raisonnable (pas dans le futur, pas trop ancienne).

## Considérations de portabilité

- La même clé fonctionne pour signer les entries ET pour s'auth sur un chat-server.
- Compatible `npub` Nostr → un user Nostr peut directement contribuer avec sa clé existante.
- Si l'utilisateur perd sa clé : il en génère une nouvelle. Ses anciennes contributions restent valides (signées par l'ancienne clé), mais il ne peut plus les modifier. Conséquence : *toujours faire un backup* (l'onboarding insiste là-dessus).

## Limitations connues

- Pas de rotation de clé natural (changer de clé = changer d'identité). Une feature future pourrait introduire un système de **chained identity** (l'ancienne clé signe la nouvelle pour prouver la continuité).
- Pas de révocation centralisée. La révocation d'une clé compromise repose sur la diffusion par les commus.
- Pas d'anonymat fort. La pubkey est traçable. Pour de l'anonymat, il faut combiner avec Tor / VPN au niveau réseau.
