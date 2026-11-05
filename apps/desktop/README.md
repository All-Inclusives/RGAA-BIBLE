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

## Préférences d’affichage

Le panneau **Affichage** permet de choisir une ambiance claire, sombre ou sépia, une palette de repères bleu et ambre, une couleur d’accentuation et une typographie (police système, Andika, Atkinson Hyperlegible, Lexend, Luciole, OpenDyslexic ou Ubuntu). Les choix sont conservés localement. La teinte utilisée pour le texte d’accentuation est ajustée pour maintenir un contraste lisible. La palette bleu et ambre est une alternative de repérage, pas une simulation du daltonisme ni une garantie adaptée à toutes les formes de déficience de la vision des couleurs.

Les fontes WOFF2 utilisées sont embarquées pour le fonctionnement hors ligne. Les fichiers de polices système du dossier ne sont pas redistribués ; le choix « Police système » utilise les fontes installées sur l’appareil. Les familles et licences documentées sont recensées dans [`assets/fonts/readme.md`](../../assets/fonts/readme.md). La licence MIT à la racine couvre le code AccessiGuide, pas le corpus ni les fontes tierces ; voir [`THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md). La palette et les réglages de contraste doivent encore être validés avec des personnes concernées et des technologies d’assistance.

Le logo ALL-Inclusive ouvre `https://www.all-inclusive.ovh/`. Le logo et l’avatar affichés dans l’application proviennent de `assets/images`.
Le pied de page affiche l’adresse de contact `bruno.ecard@gmail.com`, activable par lien e-mail, avec une taille de texte adaptée à la lecture.
