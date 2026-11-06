# AccessiGuide Desktop

Application Windows construite avec Tauri 2, TypeScript et Vite.

## Développement

Depuis `apps/desktop` :

```powershell
npm install
npm run dev
```

Le serveur Vite affiche l’aperçu à `http://localhost:5173`. Le corpus de développement est servi depuis `data/rgaa-4.1.2.json`.

## Build Windows

```powershell
npm run tauri build
```

Les installateurs MSI et NSIS sont générés sous `src-tauri/target/release/bundle`. L’application desktop charge le corpus RGAA embarqué et reste consultable hors ligne.

Pour installer le build local, ouvrez le fichier MSI dans `src-tauri/target/release/bundle/msi` ou le programme d’installation NSIS dans `src-tauri/target/release/bundle/nsis`. Ces paquets x64 sont des artefacts de développement ; ils ne sont pas signés pour une distribution publique.

## Données et mises à jour

Le corpus RGAA 4.1.2 est embarqué dans l’application. La recherche et les fiches restent disponibles sans connexion. Dans l’application Windows, **Vérifier les mises à jour** recherche la dernière Release `rgaa-data-<séquence>` du dépôt GitHub, vérifie la signature Ed25519 du manifeste puis l’intégrité et la provenance du corpus avant installation. Une mise à jour valide est activée immédiatement ; le corpus embarqué reste disponible si aucune mise à jour locale n’existe.

Le commanditaire a validé le fonctionnement des mises à jour distantes sur l’application Windows le 5 novembre 2026. Pour configurer une nouvelle build ou renouveler la clé, la clé publique Ed25519 (32 octets en 64 caractères hexadécimaux) est injectée à la compilation dans `A11Y_UPDATE_PUBLIC_KEY`, par exemple dans PowerShell avant le build :

```powershell
$env:A11Y_UPDATE_PUBLIC_KEY = "<64 caractères hexadécimaux de la clé publique>"
npm run tauri build
```

La clé privée PEM correspondante doit être conservée uniquement comme secret GitHub Actions `A11Y_UPDATE_PRIVATE_KEY` pour publier une Release avec le workflow existant. Ne placez jamais la clé privée dans le dépôt ni dans les paramètres de build. L’application n’accepte aucun téléchargement non signé. Les mises à jour ultérieures et leur contenu devront continuer à être testés au fil des publications, notamment lors de l’enrichissement pédagogique des critères.

La provenance des données et les licences du corpus et des polices tierces sont décrites dans [`THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md). Pour régénérer le corpus depuis le dépôt source officiel déjà cloné à la racine du projet, exécutez `node tools/import-rgaa.js`.

Les contenus pédagogiques sont préparés séparément du texte officiel dans [`data/pedagogy-fr.json`](../../data/pedagogy-fr.json), puis fusionnés au corpus par le même import. Ils sont liés aux tests par leur identifiant RGAA complet. Le pilote du test 1.1.1 présente quatre cas, avec code et une ou plusieurs ressources ; l’alternative, la provenance et la licence sont associées individuellement à chaque ressource. Son explication est un brouillon en langage clair, pas un contenu FALC validé. Les illustrations référencées sont fournies sous [`data/pedagogy/`](../../data/pedagogy/) et incluses par la configuration Vite ; chaque fichier référencé doit exister et avoir un nom sûr.

## Vérifications et contribution

Depuis la racine du dépôt, lancez les tests du cœur Rust :

```powershell
cargo test --workspace
```

Depuis `apps/desktop`, validez le frontend et générez les paquets Windows :

```powershell
npm run build
npm run tauri build
```

Pour contribuer, gardez séparés le texte normatif officiel et les contenus éditoriaux, conservez la provenance des données et leurs mentions de licence, puis vérifiez les changements avec les commandes ci-dessus. Les préférences et la revue d’accessibilité doivent également être testées lors de toute modification de l’interface.

## Préférences d’affichage

Le panneau **Affichage** permet de choisir une ambiance claire, sombre ou sépia, une palette de repérage (classique, bleu et ambre, ou un des sept profils deutéranomalie, deutéranopie, protanomalie, protanopie, tritanomalie, tritanopie et achromatopsie), une couleur d’accentuation et une typographie (police système, Andika, Atkinson Hyperlegible, Lexend, Luciole, OpenDyslexic ou Ubuntu). Les choix sont conservés localement. Les profils ajustent les couleurs d’accent et de repérage, sans simuler la vision ni garantir une réponse universelle ; les libellés et symboles ne reposent pas sur la couleur seule. La palette bleu et ambre a été validée par des personnes concernées ; la validation des sept nouveaux profils reste à faire.

Les fontes WOFF2 utilisées sont embarquées pour le fonctionnement hors ligne. Les fichiers de polices système du dossier ne sont pas redistribués ; le choix « Police système » utilise les fontes installées sur l’appareil. Les familles et licences documentées sont recensées dans [`assets/fonts/readme.md`](../../assets/fonts/readme.md). La licence MIT à la racine couvre le code AccessiGuide, pas le corpus ni les fontes tierces ; voir [`THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md). La palette et les réglages de contraste doivent encore être validés avec des personnes concernées et des technologies d’assistance.

Le logo ALL-Inclusive ouvre `https://www.all-inclusive.ovh/`. Le logo et l’avatar affichés dans l’application proviennent de `assets/images`.
Le pied de page affiche l’adresse de contact `bruno.ecard@gmail.com`, activable par lien e-mail, avec une taille de texte adaptée à la lecture.
