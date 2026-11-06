# Plan de développement — A11y-Bible / AccessiGuide Dev

> Document de pilotage vivant : il fixe les étapes et les critères de validation du projet. Il sera mis à jour à chaque jalon, décision importante ou changement de périmètre.

**Version :** 2.0 — **Date :** 6 novembre 2026
**Statut général :** jalons 0 à 4 validés ; mise à jour distante signée du corpus RGAA testée avec succès. Un canal Tauri distinct pour les mises à jour de l’application est en cours de préparation ; il nécessite les secrets de signature et une première installation manuelle de la version bootstrap. Le modèle pédagogique est ajouté et le pilote 1.1.1 est disponible, ses textes restant des brouillons à relire.
**Cahier des charges :** `Cahier des charges RGAA BIBLE .txt`

## 1. Vision et objectif

Construire une application multiplateforme qui aide les développeurs, designers et auditeurs à comprendre et appliquer les règles d’accessibilité numérique. Elle doit rendre consultables les références normatives, les expliquer en langage clair et montrer leurs conséquences concrètes pour les personnes concernées.

Le produit doit fonctionner hors ligne. Son moteur métier et ses données seront séparés de l’interface afin de permettre, à terme, d’autres intégrations (par exemple une extension IDE ou une CLI).

## 2. Source de référence

- **Source RGAA retenue :** dépôt officiel indiqué par le commanditaire : [`DISIC/accessibilite.numerique.gouv.fr`](https://github.com/DISIC/accessibilite.numerique.gouv.fr).
- **Commande communiquée :** `gh repo clone DISIC/accessibilite.numerique.gouv.fr`
- **Clone local d’inventaire :** `accessibilite.numerique.gouv.fr\`, à la racine du projet. Le dépôt a été récupéré avec Git (la commande `gh` n’est pas installée dans l’environnement).
- **Révision examinée :** `ca4019f95073b6cbd2482a16e9f12b52d8de678d`, commit du 18 août 2026 sur `main` au moment de l’inventaire.
- **Licence indiquée par le dépôt :** code du site sous MIT ; contenus sous Licence Ouverte 2.0, sauf droits de tiers. Le dépôt précise également des restrictions sur la marque de l’État et la typographie Marianne. Avant redistribution, conserver l’attribution et vérifier les mentions propres aux fichiers ou ressources concernés.
- **Aucun lien supplémentaire n’est requis pour amorcer le MVP RGAA.** Le dépôt référence WCAG 2.1 et fournit les PDF français et anglais de la norme EN 301 549 V2.1.2. Ces documents sont des références datées de 2018 ; vérifier la version pertinente avant d’élargir le périmètre normatif.
- Conserver la provenance et la version de chaque donnée importée. Distinguer strictement les textes normatifs des explications pédagogiques, impacts usagers et exemples éditoriaux.

### Inventaire initial de la source (4 novembre 2026)

| Élément | Constat |
|---|---|
| Version RGAA courante | Le README indique que le RGAA 4.1.2 est publié le 18 avril 2023 et que ses données sont à la racine de `RGAA\`. |
| Version archivée | `RGAA\4.1\` contient les fichiers de la version 4.1. Ne pas confondre ce dossier avec la version courante 4.1.2. |
| Critères | `RGAA\criteres.json` décrit 13 thèmes et 106 critères, leurs tests et des références WCAG 2.1. |
| Méthodologies | `RGAA\methodologies.json` contient 258 entrées indexées par identifiant de test. |
| Glossaire | `RGAA\glossaire.json` contient 119 définitions avec du contenu HTML. |
| Référentiels liés | Le JSON des critères expose des références WCAG, mais pas de champ EN 301 549 dans ce fichier. Le dépôt contient des pages de référence et les PDF EN 301 549 V2.1.2 en français et en anglais. |
| Limites des données | Le JSON ne fournit pas les filtres par handicap ou composant, ni les impacts usagers, les explications métier ou les exemples de code attendus par le produit. Ces contenus devront être éditoriaux, sourcés et clairement séparés des données officielles. |
| Portée normative | L’introduction du dépôt précise que la méthode technique RGAA vise les pages Web et ne couvre pas les applications mobiles natives ; celles-ci nécessitent notamment une analyse directe de la norme EN 301 549. |

**Conséquence pour l’import :** utiliser les JSON générés par le dépôt comme source structurée initiale, conserver la version et la révision source, et traiter prudemment le HTML intégré aux définitions. Ne pas importer l’ensemble du site, ses dépendances ou ses éléments de marque dans l’application.

### Modèle et import validés au jalon 1

- **Schéma :** `schema/rgaa-dataset.schema.json`, version `1.0.0`.
- **Corpus importé :** `data/rgaa-4.1.2.json`. Les fichiers normatifs sont conservés dans le champ `official` ; les textes pédagogiques, situations de handicap, composants et exemples sont des champs éditoriaux séparés et vides par défaut.
- **Identifiants :** identifiants locaux sous la forme `1.1`, avec une référence explicite au référentiel/version au niveau des métadonnées source ; ils ne sont pas présentés comme des identifiants universels entre référentiels.
- **Correspondances :** conserver les références WCAG présentes dans la source officielle ; ne pas créer de correspondance EN 301 549 tant qu’elle n’a pas été vérifiée et documentée.
- **Provenance :** révision Git complète et SHA-256 de chacun des trois JSON sources intégrés au manifeste du corpus.
- **Glossaire :** le HTML officiel est conservé comme texte source (`bodyHtml`). Il devra être assaini avant tout rendu dans une interface ; ne pas l’injecter directement comme HTML de confiance.
- **Stockage MVP :** JSON versionné embarqué, retenu provisoirement vu le volume importé (~543 Ko) et le besoin d’un corpus local simple. Réévaluer si la recherche, la taille du corpus ou le cycle de mise à jour le justifie.
- **Reproduction :** depuis la racine du projet, exécuter `node tools/import-rgaa.js`. Un chemin de dépôt source différent peut être passé en premier argument. Le script valide les données sources, l’unicité des critères et tests, la couverture exacte des méthodologies et la structure importée avant d’écrire le JSON de sortie.

### Structure des contenus pédagogiques — pilote

- **Grille de travail :** `data/pedagogy-fr.json`, rattachée aux tests normatifs par leur identifiant complet (par exemple `1.1.1`). L’import officiel reste reproductible ; la grille éditoriale est fusionnée séparément dans le corpus généré.
- **Champs :** état de relecture, explication en langage clair (brouillon distinct du texte officiel), bénéfice usager et cas illustrés. Chaque cas peut référencer plusieurs ressources, chacune avec identifiant, nom de fichier, texte alternatif (`null` = attribut absent, chaîne vide = alternative vide), provenance et licence ; le code du cas est conservé séparément.
- **Affichage :** la fiche montre les cas sous forme de cartes pédagogiques, et non le tableau éditorial de préparation. Le test `1.1.1` sert de pilote avec une image décorative, une alternative absente, une alternative présente mais insuffisante et une alternative pertinente. L’illustration floutée est un effet de présentation ; le défaut normatif démontré est l’absence d’alternative.
- **Validation :** les textes du pilote sont des brouillons, pas du FALC validé. Le corpus valide que chaque entrée référence un test existant et que les champs et ressources sont renseignés. Les illustrations sont servies depuis `data/pedagogy/` par le répertoire public de Vite ; leur nom de fichier doit être sûr et chaque fichier référencé doit être inclus dans le build.

### Cœur métier validé au jalon 2

- **Crate Rust :** `crates/a11y-core`, membre du workspace Cargo à la racine.
- **Chargement :** corpus embarqué, chaîne JSON ou chemin local ; validation de la provenance Git, des identifiants, des compteurs et de la correspondance exhaustive tests/méthodologies.
- **API :** `query_articles`, `get_criterion`, `filter_by_disability`, `get_code_examples` et `check_data_version`. Les types publics sont documentés dans la crate.
- **Recherche :** texte normatif et éditorial, filtres combinables, identifiant ou titre de thème, classement déterministe et correspondance insensible à la casse et aux accents. Les exemples pédagogiques restent vides tant qu’ils ne sont pas rédigés.
- **Performance :** le texte normalisé des critères est indexé au chargement ; sur le corpus courant, le benchmark Release de 5 000 recherches « alternative textuelle » est passé d’environ 2,72 ms à 63,5 µs par recherche.
- **Validation :** 7 tests unitaires couvrent le chargement, la recherche, les filtres, la version, les exemples et les erreurs de données.

### Mises à jour locales validées au jalon 3

- **Manifeste :** `schema/rgaa-update-manifest.schema.json`, version de schéma `1.0.0`, numéro de séquence monotone, version RGAA, révision source, nom de paquet, empreinte SHA-256 et taille en octets.
- **Compatibilité des versions :** le numéro du manifeste de mise à jour (`1.0.0`) est indépendant de la version du corpus (`1.1.0`). Le cœur accepte les corpus `1.0.0` et `1.1.0` ; les champs pédagogiques sont optionnels à la lecture des anciens corpus.
- **Paquet de publication :** généré par `node tools/package-rgaa.js <sequence>` sous `releases/rgaa-4.1.2/<sequence>/`. La sortie est construite dans un dossier temporaire puis renommée ; le script refuse d’écraser une séquence existante.
- **Validation et activation :** `install_update` vérifie le manifeste, la taille et l’empreinte, la provenance, la structure du corpus et les compteurs ; il écrit en staging sur le même volume puis active par renommage atomique du dossier versionné. `load_or_bundled` charge la version locale la plus récente ou le corpus intégré s’il n’y a pas encore de mise à jour.
- **Reprise :** les dossiers `.staging-*` incomplets sont ignorés ; les paquets invalides ne remplacent pas la version active ; les séquences répétées identiques sont idempotentes et les retours à une séquence antérieure sont refusés.
- **Signature :** les manifestes peuvent être signés Ed25519 ; `install_signed_update` et `load_signed_or_bundled` exigent une clé publique de confiance. Le générateur signe si `A11Y_UPDATE_PRIVATE_KEY` contient une clé privée PEM, et refuse les publications sans signature avec `--require-signature`.
- **Publication :** workflow manuel `.github/workflows/publish-rgaa-dataset.yml`, séquence explicite et publication comme Release GitHub taguée `rgaa-data-<sequence>`. Le dépôt public [`All-Inclusives/RGAA-BIBLE`](https://github.com/All-Inclusives/RGAA-BIBLE) a reçu le commit initial `5dd396ee7ca8997cc958677622517f3fcba1f953`. Le commanditaire confirme qu’une mise à jour distante signée a été testée avec succès sur l’application Windows le 5 novembre 2026.
- **Sécurité :** le paquet de séquence 1 du dépôt est un artefact local de test non signé ; il ne doit pas être publié. La clé privée ne doit jamais être stockée dans le projet et reste réservée au secret GitHub Actions `A11Y_UPDATE_PRIVATE_KEY` ; seule la clé publique correspondante est intégrée à l’application. La validation à chaque publication de données ultérieure demeure nécessaire.

### Mises à jour de l’application desktop

- **Canal séparé :** le plugin officiel Tauri Updater contrôle et installe l’application signée ; le client RGAA conserve son canal `rgaa-data-<séquence>`. Les deux contrôles ont des statuts distincts dans l’interface.
- **Publication :** le workflow `.github/workflows/publish-desktop.yml` construit NSIS signé, publie une Release `accessiguide-v<version>` et actualise le manifeste `latest.json` dans la Release stable `accessiguide-update-feed`.
- **Clés :** la clé publique Tauri est intégrée à la configuration ; la clé privée correspondante doit être conservée comme secret `TAURI_SIGNING_PRIVATE_KEY`, hors du dépôt. Le flux de corpus nécessite également `A11Y_UPDATE_PUBLIC_KEY` pour les builds de publication.
- **Bootstrap :** la version précédente 0.1.0 ne comporte pas le plugin updater. La version 0.1.1 doit être installée manuellement avant que les mises à jour de l’application puissent être automatiques. Le parcours de publication et d’installation distante reste à valider après configuration des secrets.

## 3. Principes de réalisation

1. **Données fiables et traçables :** chaque référence doit renvoyer à sa source et à sa version.
2. **Offline first :** consultation et recherche opérationnelles sans réseau ; le réseau ne sert qu’à vérifier ou télécharger les mises à jour.
3. **Séparation des responsabilités :** le cœur Rust porte le modèle, la recherche et l’accès aux données ; l’interface ne réimplémente pas ces règles métier.
4. **Accessibilité dès la conception :** navigation clavier, sémantique, annonces pour technologies d’assistance, contraste et tests intégrés à chaque étape UI.
5. **Périmètre progressif :** valider le modèle et le parcours avec un petit jeu de données avant de constituer un corpus complet.
6. **Évolution maîtrisée :** versions de schéma et de corpus explicites ; toute mise à jour est validée avant remplacement des données locales.

## 4. Périmètre

### Première version (MVP)

- Corpus officiel complet RGAA 4.1.2 : les 106 critères, leurs 258 entrées de méthodologie de test et les 119 définitions du glossaire, après validation de l’import.
- Recherche par référence ou texte, et filtres par composant/besoin technique et situation de handicap.
- Fiche de critère avec référence officielle ; les explications, impacts usagers et exemples de code sont des contenus éditoriaux à enrichir progressivement, sans les confondre avec le corpus normatif.
- Français comme langue complète de départ ; modèle prêt à accueillir l’anglais sans inventer des traductions absentes.
- Fonctionnement hors ligne avec données locales.
- Application Tauri Windows desktop en première cible ; autres plateformes différées, sans les exclure de l’architecture.
- Tests du cœur, des recherches, de l’import des données et des parcours essentiels.

### Préparé, mais hors du premier livrable

- Corpus complet et maintenu pour WCAG et EN 301 549 : définir les sources et les liens de correspondance après l’audit des sources.
- Traduction anglaise complète et autres langues.
- Synchronisation distante automatique, signature et distribution de jeux de données.
- Extensions VS Code/JetBrains, serveur LSP, CLI ou daemon.
- Audit externe de conformité et certification éventuelle.

## 5. Jalons de développement

Les semaines ci-dessous reprennent l’ordre indicatif du cahier des charges. Elles ne constituent pas une estimation ferme ; la durée sera réévaluée après l’inventaire des sources et la validation du modèle de données.

### Jalon 0 — Cadrage et état initial

**Objectif :** rendre le projet exécutable et confirmer les choix avant d’engager l’architecture.

- [x] Inventorier les fichiers du projet et l’environnement de développement.
- [x] Examiner le dépôt officiel, son organisation, les formats, les versions et les conditions de réutilisation.
- [x] Confirmer le périmètre : corpus officiel RGAA 4.1.2 complet, enrichissement pédagogique progressif, Windows desktop comme première cible.
- [x] Définir les critères de succès du MVP (section 6).

**Livrables :** inventaire de la source, périmètre MVP et première cible consignés dans ce plan.  
**Sortie du jalon :** la source RGAA, le corpus initial et la plateforme de première livraison sont confirmés.

### Jalon 1 — Modèle de données et preuve de concept

**Objectif :** démontrer qu’un critère réel peut être importé, représenté et consulté sans perte de provenance.

- [x] Définir le schéma versionné : identifiants locaux, références, titre, version/source, langue et champs éditoriaux.
- [x] Distinguer les champs officiels des contenus pédagogiques (explications, impacts, tags et exemples).
- [x] Documenter les relations RGAA/WCAG/EN 301 549 sans supposer qu’elles sont bijectives ni fabriquer de correspondances non vérifiées.
- [x] Importer les fichiers officiels complets et vérifier la fidélité de la structure, les comptes et les liens entre critères, tests et méthodologies.
- [x] Ajouter validation et erreurs explicites en cas de source absente, invalide ou incohérente.
- [x] Choisir le JSON versionné comme stockage initial du MVP ; réévaluer cette décision si les besoins de recherche, de mise à jour ou de packaging l’exigent.

**Livrables :** schéma versionné, corpus complet traçable, import reproductible et contrôles intégrés.  
**Sortie du jalon :** les 106 critères, 258 tests, 119 définitions et 258 méthodologies sont importés ; la provenance est enregistrée et les méthodologies couvrent exactement les tests.

### Jalon 2 — Cœur métier Rust

**Objectif :** fournir une API stable de consultation et de recherche indépendante de l’interface.

- [x] Créer la crate `a11y-core` et définir les types publics.
- [x] Implémenter les fonctions de consultation par référence, recherche textuelle et filtres.
- [x] Prévoir les opérations envisagées : `query_articles`, `filter_by_disability`, `get_code_examples`, `check_data_version`.
- [x] Définir un tri stable, la gestion des accents et la combinaison des filtres.
- [x] Tester cas usuels, résultats vides, entrées invalides, références absentes et données malformées.
- [x] Mesurer la recherche sur le corpus réel, puis optimiser l’index normalisé et re-mesurer.

**Livrables :** crate compilable, API documentée, tests unitaires et corpus de test.  
**Sortie du jalon :** les mêmes requêtes donnent des résultats déterministes et correctement filtrés ; la recherche du corpus est mesurée en build optimisé.

### Jalon 3 — Données locales et synchronisation

**Objectif :** maintenir un corpus versionné consultable hors ligne et actualisable sans recompiler l’application.

- [x] Définir le manifeste versionné et les métadonnées d’intégrité et de provenance.
- [x] Définir la compatibilité, les séquences de mise à jour et l’activation d’une version locale.
- [x] Valider le paquet téléchargé (remis au cœur sous forme d’octets) en staging avant son activation.
- [x] Ne pas remplacer la dernière base valide par un paquet invalide ou un staging incomplet.
- [x] Rendre explicites les erreurs et charger le corpus intégré hors ligne si aucune mise à jour n’est installée.
- [x] Tester les paquets identiques, incompatibles, corrompus, les régressions de séquence et les dossiers staging interrompus.
- [x] Définir le mécanisme de signature Ed25519 ; rendre les mises à jour signées obligatoires pour le parcours de chargement prévu en production.
- [x] Préparer un workflow manuel GitHub Actions qui génère un paquet signé et publie les assets sous un tag de Release déterministe.
- [x] Initialiser le dépôt GitHub puis y publier le code du projet et le workflow ; vérifier le commit distant et la disponibilité du workflow.
- [x] Générer une paire de clés hors du dépôt, enregistrer la clé privée comme secret GitHub Actions et configurer uniquement la clé publique dans l’application (configuration et fonctionnement confirmés par le commanditaire).
- [x] Raccorder la découverte/téléchargement des Releases au client Tauri ; exiger une signature valide avant installation.
- [x] Valider avec succès le parcours de mise à jour distante signée dans l’application Windows (test rapporté par le commanditaire le 5 novembre 2026).

**Livrables réalisés :** formats de manifeste et paquet, générateur, vérification Ed25519, installation locale versionnée, client Tauri et test de mise à jour distante signée réussi.
**Suivi :** le parcours opérationnel est validé. Continuer à vérifier les mises à jour ultérieures et le corpus qui les accompagne, notamment au fur et à mesure de l’ajout des contenus pédagogiques.

### Jalon 4 — Interface Tauri

**Objectif :** permettre les parcours essentiels à partir de l’API métier.

- [x] Confirmer Tauri v2, TypeScript/Vite vanilla et Windows desktop comme première cible.
- [x] Créer les vues de recherche, filtre par thème, tri et résultats pour le corpus complet.
- [x] Créer la fiche détaillée avec tests, méthodologies et références normatives, ainsi que les états de chargement, erreur et absence de résultat.
- [x] Relier l’interface au cœur par les commandes Tauri `list_topics`, `query_articles` et `get_criterion`.
- [x] Construire une mise en page responsive, avec navigation clavier de base, annonces de recherche et fermeture du dialogue par Échap.
- [x] Vérifier le build et le démarrage de l’application Windows x64 ; le cœur charge le corpus embarqué hors ligne.

**Livrables en cours :** parcours MVP, intégration cœur/UI, aperçu navigateur et installateurs Windows x64 (MSI et NSIS) dans `apps/desktop/src-tauri/target/release/bundle`.
**Validation :** `npm run build`, `cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml` et `npm run tauri build` passent. Le lancement de l’exécutable Tauri est vérifié ; le cœur charge son corpus embarqué via `A11yCore::from_bundled`. Dans l’aperçu, 106 critères se chargent, la recherche « contraste » renvoie 3 résultats et la fiche s’ouvre. À 320 px, aucun débordement horizontal après correction ; au clavier, le dialogue reçoit le focus, Échap le ferme et le focus revient à la carte. Recette manuelle du commanditaire sur l’application desktop : clavier, recherches accentuées et par référence, filtres, fiches, tests, réduction maximale de la fenêtre et fonctionnement hors ligne validés.

### Jalon 5 — Accessibilité, recette et publication

**Objectif :** détecter et corriger les obstacles d’accessibilité et fiabiliser la livraison.

**Périmètre ajouté au MVP avec accord du commanditaire (4 novembre 2026) :** intégrer progressivement des préférences d’affichage utiles aux publics avec TND et aux démonstrations d’accessibilité : choix de police locale (candidats cités : OpenDyslexic, Lexend, Andika et Arial), thèmes clair/sombre/sépia, palettes pensées pour différentes formes de daltonisme et couleur d’accent réglable. Les préférences seront persistées localement. Avant d’intégrer les polices, inventorier le dossier fourni par le commanditaire et vérifier formats, licences et solutions de repli. Concevoir les palettes avec des repères autres que la couleur seule et maintenir des contrastes vérifiés ; ne pas assimiler l’option daltonisme à un filtre universel.

**Inventaire initial des polices (4 novembre 2026) :** `assets/fonts/` contient 96 fichiers WOFF2 : Andika, Atkinson Hyperlegible (dont Next et Mono), Lexend, Luciole, OpenDyslexic (dont Alta et Mono), Ubuntu et des variantes de polices système (Arial, Calibri, Comic Sans MS, Trebuchet MS, Verdana). Les licences des familles libres ont été confirmées par le commanditaire pour cet usage. Les fontes système ne sont pas redistribuées ; le choix système utilise celles installées sur le poste.

**Matrice de recette d’accessibilité (5 novembre 2026) :**

| Axe | Vérifications et critères de réussite | État |
|---|---|---|
| Clavier et focus | Tous les contrôles sont atteignables ; indicateur de focus visible ; les dialogues se ferment par Échap et restituent le focus. | Aperçu navigateur validé ; indicateur de 3 px ajouté au formulaire de recherche. |
| Lecteurs d’écran | Titres, régions, noms des contrôles et annonces de résultats, préférences et erreurs sont compréhensibles avec NVDA et Narrateur Windows. | Retest NVDA et Narrateur réussi selon le commanditaire le 5 novembre 2026. |
| Zoom et responsive | À 200 % et à 320 px CSS, contenu et commandes restent utilisables sans défilement horizontal. | Commanditaire : zoom dans l’affichage interne testé à 200 % ; loupe Windows à 200 % également validée. Dans l’aperçu, largeurs CSS 640 px et 320 px contrôlées sans débordement. Vérifier les fiches au fil de leur enrichissement, car un défaut propre à un article reste possible. |
| Contraste | Texte ≥ 4,5:1 ; indicateurs et composants non textuels ≥ 3:1 dans les trois thèmes et avec un accent personnalisé. | Profils de daltonisme fonctionnels ; contrastes sépia et sombre testés avec des personnes concernées. Contraste du focus clavier mesuré dans l’aperçu : 7,12:1 (clair), 9,40:1 (sombre), 8,02:1 (sépia). Contrôle détaillé de toutes les fiches à poursuivre progressivement. |
| Parcours et erreurs | Recherche, filtres, ouverture/fermeture de fiche, préférences, états vides et erreurs sont utilisables et annoncés. | Recherche, dialogue, préférences et erreur HTTP 500 vérifiés dans l’aperçu ; hors ligne validé sur l’application Windows par le commanditaire. |
| Hors ligne | Le corpus embarqué reste disponible sans réseau, sans altérer la connexion de l’utilisateur. | Commanditaire : fonctionnement hors ligne pleinement validé sur l’application Windows le 5 novembre 2026. |

- [x] Définir une matrice de tests clavier, lecteur d’écran, zoom, contraste et responsive.
- [x] Tester avec les technologies d’assistance pertinentes pour les plateformes effectivement prises en charge : NVDA et Narrateur Windows validés sur Windows ; VoiceOver et TalkBack ne sont pas concernés par la cible desktop actuelle.
- [x] Effectuer les tests automatisés disponibles, puis une revue manuelle : smoke test Playwright de la langue, du titre principal, des noms de contrôles, textes alternatifs, identifiants, dialogues et débordements à 640/320 px ; complété par les retests manuels du commanditaire. Aucun scanner WCAG dédié n’est configuré ; ces contrôles ne prouvent pas à eux seuls la conformité.
- [x] Vérifier le contraste des textes et contrôles principaux dans les trois thèmes et avec un accent personnalisé ; seuil mesuré supérieur ou égal à 4,5:1 dans les cas contrôlés.
- [x] Vérifier les parcours critiques et les annonces d’état, les titres, le focus et les messages d’erreur.
- [x] Traiter les anomalies bloquantes et documenter les limites connues.
- [x] Inventorier les polices fournies et implémenter les préférences locales de thème, accent et typographie avec un garde-fou de contraste ; les fontes libres utilisées sont embarquées et persistées localement.
- [x] Tester avec des personnes concernées les profils de repérage et les contrastes sépia/sombre, sans transmettre d’information par la couleur seule. Les profils ne sont pas des simulations universelles du daltonisme ; poursuivre les vérifications détaillées des fiches au fil de leur enrichissement.
- [x] Vérifier que les préférences restent appliquées après redémarrage de l’application desktop.
- [x] N’embarquer que les fichiers de polices dont les droits d’utilisation ont été confirmés ; ne pas redistribuer les fontes système.
- [x] Générer les builds et documenter installation, mise à jour, sources de données et contribution dans `apps/desktop/README.md`. Le parcours de mise à jour distante signée est testé ; les installateurs Windows restent des artefacts locaux non signés pour distribution publique.

**Livrables :** rapport de recette, anomalies traitées ou suivies, documentation utilisateur et technique, builds ciblés.  
**Sortie du jalon :** les parcours essentiels passent les tests définis et les builds ciblés sont reproductibles.

### Jalon 6 — Élargissement et interopérabilité

**Objectif :** enrichir le corpus et réutiliser le cœur au-delà de l’application autonome.

- [ ] Ajouter les autres référentiels selon la disponibilité de sources identifiées et vérifiées.
- [ ] Compléter et relire les contenus anglais ; conserver les contenus indisponibles comme tels plutôt que de présenter une traduction supposée.
- [ ] Évaluer les options d’export Rust vers WASM, CLI ou service local selon les besoins réels des intégrations IDE.
- [ ] Stabiliser une API publique et sa politique de compatibilité avant de développer une extension.

**Livrables :** corpus étendu, contrats d’API versionnés et prototype d’intégration choisi selon la demande.  
**Sortie du jalon :** une intégration consomme le cœur sans dupliquer sa logique métier.

## 6. Critères d’acceptation transversaux

- Chaque contenu normatif possède une référence, une version et une provenance vérifiables.
- Les données pédagogiques sont identifiées séparément et peuvent être révisées sans altérer la citation officielle.
- Recherche et filtres sont testés, reproductibles et documentés.
- L’application reste consultable sans connexion et préserve le dernier corpus valide après une erreur de mise à jour.
- L’usage au clavier est possible sur les parcours essentiels ; les obstacles connus des lecteurs d’écran sont consignés et traités.
- Les tests du cœur et les validations du corpus peuvent être relancés de manière reproductible.
- Les plateformes réellement livrées sont annoncées explicitement ; une cible n’est pas déclarée prise en charge tant qu’elle n’a pas été testée.

## 7. Risques et points à surveiller

| Risque | Réponse prévue |
|---|---|
| Structure ou couverture du dépôt source évolutive | Épingler la révision source et valider chaque import contre le schéma avant activation. |
| Conditions de réutilisation ou attribution mal comprises | Les vérifier avant redistribution et consigner les obligations applicables. |
| Confusion entre règle officielle et conseil pédagogique | Séparer les champs, indiquer les sources et faire relire les contenus éditoriaux. |
| Correspondances RGAA/WCAG/EN 301 549 ambiguës | Représenter les relations comme des liens documentés, pas comme une équivalence implicite. |
| Mise à jour invalide ou connexion indisponible | Valider avant activation et conserver la dernière version locale valide. |
| Extension mobile coûteuse ou hors portée des critères RGAA Web | La différer après Windows desktop et auditer séparément les exigences EN 301 549 applicables. |
| Accessibilité traitée uniquement en fin de projet | Ajouter des critères et tests manuels à chaque étape UI, puis réaliser une recette dédiée. |

## 8. Suivi du plan

- Mettre à jour les cases à cocher uniquement après vérification du livrable correspondant.
- À chaque fin de jalon, noter ici la date, le résultat, les écarts et les décisions prises.
- Si le périmètre ou une hypothèse structurante change, mettre à jour les sections concernées et la version du document.
- Prochaine action recommandée : commencer l’enrichissement pédagogique des critères (explications, impacts usagers, tags et exemples), exposer progressivement les filtres par composant et situation de handicap dans l’interface, et vérifier chaque fiche à 200 % ainsi que les mises à jour de données au fil des publications.

### Journal des jalons

| Date | Jalon | Résultat / décisions |
|---|---|---|
| 4 novembre 2026 | Création du plan | Plan initial établi à partir du cahier des charges ; dépôt DISIC retenu comme source RGAA. |
| 4 novembre 2026 | Inventaire initial | Dépôt cloné à la révision `ca4019f` ; RGAA 4.1.2 courant à la racine (13 thèmes, 106 critères), archive 4.1 séparée ; 119 définitions et 258 méthodologies de test. Licence et limites Web/mobile consignées. |
| 4 novembre 2026 | Cadrage MVP | Périmètre confirmé : corpus officiel RGAA 4.1.2 complet avec enrichissement pédagogique progressif ; Windows desktop comme première cible. |
| 4 novembre 2026 | Jalon 1 terminé | Schéma `1.0.0`, import reproductible et corpus JSON versionné créés. Validation complète : 13 thèmes, 106 critères, 258 tests, 119 définitions ; 258/258 méthodologies correspondantes. Réimport identique ; source inchangée. JSON retenu provisoirement pour le MVP. |
| 4 novembre 2026 | Jalon 2 terminé | Workspace Cargo et crate `a11y-core` créés. API indépendante de Tauri, recherche accent-insensible, filtres et validation de corpus implémentés. 7 tests passent ; Clippy sans avertissement ; benchmark après indexation : ~63,5 µs/recherche en Release. |
| 4 novembre 2026 | Jalon 3 — socle et pipeline local | Manifeste, paquet de test séquence 1, vérification SHA-256, installation versionnée atomique et signature Ed25519 implémentés ; 14 tests passent. Workflow GitHub Releases préparé pour `All-Inclusives/RGAA-BIBLE` ; clés et téléchargement réseau restent à configurer. |
| 4 novembre 2026 | Publication initiale GitHub | Premier commit `5dd396e` poussé sur `main` avec le code, le corpus validé et le workflow. GitHub Actions confirme le workflow « Publish RGAA dataset » actif. Aucun secret ni clé privée n’a été publié ; aucune Release de données n’a été créée. |
| 4 novembre 2026 | Reprise après crash — aperçu du jalon 4 | Après interruption, le serveur de développement a été relancé (`apps/desktop`, `npm run dev -- --host 127.0.0.1`) et l’aperçu navigateur est de nouveau accessible (HTTP 200). Vérification du corpus local : 13 thèmes, 106 critères et 258 tests ; la recherche « contraste » renvoie 3 critères et l’ouverture de la fiche 3.2 fonctionne. La recette d’accessibilité et la validation complète du jalon 4 restent à faire. |
| 4 novembre 2026 | Jalon 4 — responsive et build Windows | La recette navigateur confirme la navigation clavier du dialogue et l’absence de débordement à 320 px ; la largeur minimale mobile a été corrigée dans `apps/desktop/src/styles.css`. `npm run tauri build` génère l’exécutable Windows x64 et les paquets MSI/NSIS ; le test de démarrage de l’exécutable reste actif après 8 secondes. Le cœur Tauri initialise le corpus intégré avec `A11yCore::from_bundled`, pour la consultation hors ligne. La recette lecteur d’écran et le test interactif complet du paquet restent à faire. |
| 4 novembre 2026 | Recette manuelle desktop par le commanditaire | Parcours clavier et focus, recherche par texte et référence, filtres, consultation des fiches et fenêtres réduites au minimum validés. Le test hors ligne est reporté car des processus en cours ne peuvent pas être interrompus ; aucun changement n’est apporté à la connexion. La recette lecteur d’écran reste à faire. |
| 4 novembre 2026 | Jalon 5 — première passe d’audit et préférences TND | Le contrôle navigateur des libellés, champs, dialogue, annonces et contrastes a conduit à assombrir les petits textes insuffisamment contrastés et le placeholder, ainsi qu’à supprimer une annonce de résultats redondante. `npm run build` passe ; le parcours recherche et dialogue reste fonctionnel. Avec l’accord du commanditaire, les préférences de police, thèmes clair/sombre/sépia, palettes daltonisme et couleur d’accent entrent au périmètre du MVP en plusieurs étapes. L’audit lecteur d’écran et le test hors ligne sont encore en attente. |
| 4 novembre 2026 | Jalon 5 — inventaire des polices locales | Inventaire de `assets/fonts/` : 96 fichiers WOFF2 répartis entre Andika, Atkinson Hyperlegible/Next/Mono, Lexend, Luciole, OpenDyslexic/Alta/Mono, Ubuntu et des fontes système Microsoft. Le README donne des indications de licence, mais aucun fichier de licence distinct n’est présent ; les fontes système ne seront pas embarquées avant vérification des droits. |
| 4 novembre 2026 | Jalon 5 — première version des préférences d’affichage | Thèmes clair, sombre et sépia, palette bleu/ambre, accent personnalisé et persistance locale implémentés. Le contrôle navigateur vérifie la réapplication après rechargement, la fermeture clavier avec restitution du focus et l’affichage à 320 px ; l’audit de contraste corrigé passe dans les trois thèmes (minimum mesuré : 4,51:1). La palette alternative est explicitement documentée comme non universelle et non simulatrice du daltonisme. |
| 4 novembre 2026 | Jalon 5 — build desktop relancé | `npm run build` et `npm run tauri build` passent après intégration des préférences ; les installateurs Windows x64 MSI et NSIS ont été régénérés. L’exécutable desktop démarre et affiche sa fenêtre AccessiGuide ; la recette utilisateur des préférences après fermeture/réouverture, le lecteur d’écran et la validation de la palette restent à faire. |
| 4 novembre 2026 | Jalon 5 — recette lecteurs d’écran Windows | Le commanditaire confirme que l’application fonctionne avec NVDA et le Narrateur Windows. Le test des technologies d’assistance sur la cible Windows desktop est validé. |
| 4 novembre 2026 | Jalon 5 — préférences et contraste du mode sombre | Après redémarrage de la fenêtre desktop, le commanditaire confirme que les préférences d’apparence sont conservées et valide la palette sépia. Les libellés « Test x.x.x », « Méthodologie » et la version RGAA de l’en-tête en mode sombre utilisent désormais la couleur d’accentuation ajustée pour le contraste. |
| 4 novembre 2026 | Jalon 5 — intégration des typographies | Le commanditaire confirme les droits d’utilisation des fontes libres du dossier. Andika, Atkinson Hyperlegible, Lexend, Luciole, OpenDyslexic et Ubuntu sont intégrées à l’application desktop en variantes régulière et grasse ; le choix de typographie est persisté localement et les fontes système ne sont pas redistribuées. `npm run build` passe ; le chargement effectif des six familles et l’application des sept choix (dont la police système) sont vérifiés dans l’aperçu navigateur. |
| 4 novembre 2026 | Licence du code AccessiGuide | Le commanditaire choisit MIT pour le code AccessiGuide, avec attribution à ALL-Inclusive / Bruno ECARD. Le corpus RGAA et les polices tierces conservent leurs licences propres, documentées dans `THIRD_PARTY_NOTICES.md`. |
| 5 novembre 2026 | Jalon 5 — audit clavier et annonces | La recette navigateur a révélé que le champ de recherche neutralisait son contour de focus sans indicateur de remplacement suffisant ; un contour de 3 px avec contraste mesuré supérieur à 7:1 a été ajouté dans les trois thèmes. Les erreurs de chargement transmettent maintenant leur message complet dans la région de statut annoncée. Recherche « contraste » (3 résultats), ouverture/fermeture de fiche avec restitution du focus, préférences et Échap, erreur HTTP 500 et affichage à 320 px validés dans l’aperçu. `npm run build` et `npm run tauri build` passent ; l’exécutable et les installateurs Windows sont générés. Retest NVDA/Narrateur, zoom 200 %, vérification automatisée et recette hors ligne restent à faire. |
| 5 novembre 2026 | Jalon 5 — retests lecteurs d’écran et loupe | Le commanditaire confirme que le retest avec NVDA et le Narrateur Windows fonctionne, et que la loupe Windows est utilisable avec un grossissement à 200 %. Le zoom de mise en page à 200 % reste à vérifier séparément. |
| 5 novembre 2026 | Jalon 4 — fonctionnement hors ligne validé | Le commanditaire confirme que le fonctionnement hors ligne de l’application est pleinement fonctionnel. Le parcours hors ligne est validé sans déconnecter le PC ni interrompre les autres services. |
| 5 novembre 2026 | Jalon 3 — client de mises à jour signées | L’application Windows expose une vérification manuelle des Releases de données GitHub, limite les téléchargements, n’accepte que les assets attendus du dépôt, vérifie la signature Ed25519 avant le paquet et installe via le mécanisme atomique du cœur. Le corpus actif est remplacé seulement après validation et reste disponible hors ligne. La clé publique est injectée à la compilation par `A11Y_UPDATE_PUBLIC_KEY` ; aucune clé n’est fournie dans le dépôt et aucune Release signée n’existe encore, donc le test réseau de bout en bout attend la configuration par le commanditaire. |
| 5 novembre 2026 | Jalon 5 — profils de repérage couleur | Avec l’accord du commanditaire, sept profils (formes partielles et totales rouge-vert et bleu-jaune, et achromatopsie) sont ajoutés au sélecteur. Ce sont des palettes ajustant les accents et repères, pas des filtres de simulation ni des garanties universelles. La palette bleu et ambre initiale est validée par des personnes concernées ; la recette de contraste des nouveaux profils dans les trois thèmes et leur validation utilisateur restent à réaliser. |
| 5 novembre 2026 | Jalon 5 — recette de base et documentation | Le retest NVDA/Narrateur et la loupe Windows à 200 % sont confirmés par le commanditaire. Des smoke tests Playwright vérifient la langue, le titre principal, les noms des contrôles visibles, les textes alternatifs, l’unicité des identifiants, les noms de dialogues ouverts et l’absence de débordement horizontal à 640 et 320 px CSS. Les parcours recherche, filtres, dialogues, préférences, retour du focus et annonce d’une erreur sont exercés dans l’aperçu ; le fonctionnement hors ligne est validé sur l’exécutable par le commanditaire. Le README desktop documente maintenant l’installation locale, la disponibilité des données, l’absence actuelle de mises à jour distantes et les commandes de contribution. Limites restantes : pas de scanner WCAG dédié configuré, palette à valider avec des personnes concernées, et installateurs non signés pour la distribution publique. |
| 5 novembre 2026 | Validation des mises à jour et des profils d’affichage | Le commanditaire confirme que les mises à jour distantes ont été testées avec succès. Les profils de repérage liés aux daltonismes sont fonctionnels ; les contrastes sépia et sombre ont été testés avec des personnes concernées, ainsi que le zoom de l’affichage interne à 200 %. La robustesse sera vérifiée au fil des prochains jeux de données et des fiches pédagogiques, y compris les éventuels défauts propres à certains articles. |
| 6 novembre 2026 | Structure pédagogique et pilote 1.1.1 | Ajout du modèle structuré `pedagogicalTests`, de la grille éditoriale indépendante `data/pedagogy-fr.json` et de son affichage dans les fiches. Le test 1.1.1 est illustré par quatre cas liés à la même ressource, avec états d’alternative distincts, snippets, provenance et licence. Les textes restent explicitement en brouillon à relire. Les 16 tests Rust et le build TypeScript passent ; le navigateur confirme le chargement des illustrations et l’absence de débordement à 1280, 640 et 320 px. |
| 6 novembre 2026 | Diagnostic et préparation de l’auto-mise à jour de l’application | Confirmation que le bouton précédent ne contrôlait que les données RGAA. Ajout du plugin officiel Tauri Updater, d’un statut d’application distinct, d’un flux `latest.json` stable et d’un workflow de publication NSIS signé. L’application passe en version 0.1.1 pour le bootstrap ; l’installation et la publication de bout en bout restent à faire après configuration des secrets. |
