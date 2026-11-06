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
$env:TAURI_SIGNING_PRIVATE_KEY = "$env:USERPROFILE\.tauri\accessiguide-updater.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
npm run tauri build
```

Les installateurs MSI et NSIS sont générés sous `src-tauri/target/release/bundle`. L’application desktop charge le corpus RGAA embarqué et reste consultable hors ligne.

Pour installer le build local, ouvrez le fichier MSI dans `src-tauri/target/release/bundle/msi` ou le programme d’installation NSIS dans `src-tauri/target/release/bundle/nsis`. Ces paquets x64 sont des artefacts de développement ; leur signature Tauri sert à authentifier les mises à jour et ne remplace pas une signature Authenticode de distribution Windows.

## Mises à jour de l’application et des données

L’application possède deux mises à jour indépendantes, clairement séparées dans **Affichage** :

- **Application AccessiGuide** : le plugin officiel Tauri vérifie un manifeste distinct, puis télécharge et installe un installeur Windows NSIS signé. Une relance applique la nouvelle version. Le flux stable `accessiguide-update-feed` contient les métadonnées ; les exécutables signés sont publiés dans une Release versionnée `accessiguide-v<version>`.
- **Données du référentiel RGAA** : le bouton dédié recherche les Releases `rgaa-data-<séquence>`, vérifie la signature Ed25519 du manifeste, puis l’intégrité et la provenance du corpus avant activation immédiate. Le corpus intégré reste disponible hors ligne.

Le parcours d’application nécessite les secrets GitHub Actions `TAURI_SIGNING_PRIVATE_KEY` (contenu de la clé privée Tauri, à conserver hors du dépôt) et `A11Y_UPDATE_PUBLIC_KEY` (clé publique du canal de données, 64 caractères hexadécimaux). La clé publique Tauri utilisée pour vérifier les installeurs est intégrée à `src-tauri/tauri.conf.json` ; le secret `TAURI_SIGNING_PRIVATE_KEY` doit contenir le contenu du fichier privé, pas son chemin. Le secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` est requis seulement si la clé privée est protégée par un mot de passe.

La clé privée Tauri locale est conservée hors du dépôt sous `%USERPROFILE%\.tauri\accessiguide-updater.key`. Pour une machine de développement, définissez-la avant de générer un installeur signé :

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = "$env:USERPROFILE\.tauri\accessiguide-updater.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
$env:A11Y_UPDATE_PUBLIC_KEY = "<64 caractères hexadécimaux de la clé publique>"
npm run tauri build -- --bundles nsis
```

Le workflow manuel **Publish AccessiGuide desktop** construit et signe l’application, publie la Release versionnée et actualise le flux stable. **La version 0.1.0 ne contient pas le nouveau mécanisme d’auto-mise à jour : son remplacement initial par la version 0.1.1 doit être fait en lançant manuellement le nouvel installeur signé. Les mises à jour automatiques ne fonctionneront qu’à partir de cette version bootstrap.** Ne placez jamais une clé privée dans le dépôt. Les versions et leur contenu doivent continuer à être testés après publication.

Pour le canal de données, la clé privée Ed25519 reste uniquement dans le secret GitHub Actions `A11Y_UPDATE_PRIVATE_KEY`. Le corpus et l’application ont des séquences/versionnements distincts ; mettre à jour l’un ne remplace pas l’autre.

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
```

Pour créer un installeur, définissez d’abord les variables de signature indiquées dans « Build Windows », puis exécutez `npm run tauri build`. Pour contribuer, gardez séparés le texte normatif officiel et les contenus éditoriaux, conservez la provenance des données et leurs mentions de licence, puis vérifiez les changements avec les commandes ci-dessus. Les préférences et la revue d’accessibilité doivent également être testées lors de toute modification de l’interface.

## Préférences d’affichage

Le panneau **Affichage** permet de choisir une ambiance claire, sombre ou sépia, une palette de repérage (classique, bleu et ambre, ou un des sept profils deutéranomalie, deutéranopie, protanomalie, protanopie, tritanomalie, tritanopie et achromatopsie), une couleur d’accentuation et une typographie (police système, Andika, Atkinson Hyperlegible, Lexend, Luciole, OpenDyslexic ou Ubuntu). Les choix sont conservés localement. Les profils ajustent les couleurs d’accent et de repérage, sans simuler la vision ni garantir une réponse universelle ; les libellés et symboles ne reposent pas sur la couleur seule. La palette bleu et ambre a été validée par des personnes concernées ; la validation des sept nouveaux profils reste à faire.

Les fontes WOFF2 utilisées sont embarquées pour le fonctionnement hors ligne. Les fichiers de polices système du dossier ne sont pas redistribués ; le choix « Police système » utilise les fontes installées sur l’appareil. Les familles et licences documentées sont recensées dans [`assets/fonts/readme.md`](../../assets/fonts/readme.md). La licence MIT à la racine couvre le code AccessiGuide, pas le corpus ni les fontes tierces ; voir [`THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md). La palette et les réglages de contraste doivent encore être validés avec des personnes concernées et des technologies d’assistance.

Le logo ALL-Inclusive ouvre `https://www.all-inclusive.ovh/`. Le logo et l’avatar affichés dans l’application proviennent de `assets/images`.
Le pied de page affiche l’adresse de contact `bruno.ecard@gmail.com`, activable par lien e-mail, avec une taille de texte adaptée à la lecture.
