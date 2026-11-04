# Plan de développement — A11y-Bible / AccessiGuide Dev

> Document de pilotage vivant : il fixe les étapes et les critères de validation du projet. Il sera mis à jour à chaque jalon, décision importante ou changement de périmètre.

**Version :** 0.7 — **Date :** 4 novembre 2026  
**Statut général :** jalons 0, 1 et 2 terminés ; socle et pipeline de publication du jalon 3 prêts, dépôt à initialiser  
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

### Cœur métier validé au jalon 2

- **Crate Rust :** `crates/a11y-core`, membre du workspace Cargo à la racine.
- **Chargement :** corpus embarqué, chaîne JSON ou chemin local ; validation de la provenance Git, des identifiants, des compteurs et de la correspondance exhaustive tests/méthodologies.
- **API :** `query_articles`, `get_criterion`, `filter_by_disability`, `get_code_examples` et `check_data_version`. Les types publics sont documentés dans la crate.
- **Recherche :** texte normatif et éditorial, filtres combinables, identifiant ou titre de thème, classement déterministe et correspondance insensible à la casse et aux accents. Les exemples pédagogiques restent vides tant qu’ils ne sont pas rédigés.
- **Performance :** le texte normalisé des critères est indexé au chargement ; sur le corpus courant, le benchmark Release de 5 000 recherches « alternative textuelle » est passé d’environ 2,72 ms à 63,5 µs par recherche.
- **Validation :** 7 tests unitaires couvrent le chargement, la recherche, les filtres, la version, les exemples et les erreurs de données.

### Mises à jour locales validées au jalon 3

- **Manifeste :** `schema/rgaa-update-manifest.schema.json`, version de schéma `1.0.0`, numéro de séquence monotone, version RGAA, révision source, nom de paquet, empreinte SHA-256 et taille en octets.
- **Paquet de publication :** généré par `node tools/package-rgaa.js <sequence>` sous `releases/rgaa-4.1.2/<sequence>/`. La sortie est construite dans un dossier temporaire puis renommée ; le script refuse d’écraser une séquence existante.
- **Validation et activation :** `install_update` vérifie le manifeste, la taille et l’empreinte, la provenance, la structure du corpus et les compteurs ; il écrit en staging sur le même volume puis active par renommage atomique du dossier versionné. `load_or_bundled` charge la version locale la plus récente ou le corpus intégré s’il n’y a pas encore de mise à jour.
- **Reprise :** les dossiers `.staging-*` incomplets sont ignorés ; les paquets invalides ne remplacent pas la version active ; les séquences répétées identiques sont idempotentes et les retours à une séquence antérieure sont refusés.
- **Signature :** les manifestes peuvent être signés Ed25519 ; `install_signed_update` et `load_signed_or_bundled` exigent une clé publique de confiance. Le générateur signe si `A11Y_UPDATE_PRIVATE_KEY` contient une clé privée PEM, et refuse les publications sans signature avec `--require-signature`.
- **Publication :** workflow manuel `.github/workflows/publish-rgaa-dataset.yml`, séquence explicite et publication comme Release GitHub taguée `rgaa-data-<sequence>`. Le dépôt cible `All-Inclusives/RGAA-BIBLE` a été vérifié public et vide (aucune branche) ; le workflow est prêt localement mais n’a pas été poussé ni exécuté.
- **Sécurité :** le paquet de séquence 1 du dépôt est un artefact local de test non signé ; il ne doit pas être publié. La clé privée n’est ni générée ni stockée dans le projet. Le secret Actions `A11Y_UPDATE_PRIVATE_KEY` doit être ajouté au dépôt. L’application exigera la clé publique correspondante pour lire les mises à jour signées. L’API de téléchargement GitHub sera raccordée au consommateur Tauri.

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
- [ ] Initialiser le dépôt GitHub vide puis y publier le code du projet et le workflow.
- [ ] Générer une paire de clés hors du dépôt, enregistrer la clé privée comme secret GitHub Actions et configurer uniquement la clé publique dans l’application.
- [ ] Raccorder la découverte/téléchargement des Releases au client Tauri ; exiger une signature valide avant installation.

**Livrables réalisés :** formats de manifeste et paquet, générateur, vérification Ed25519, installation locale versionnée et tests.  
**Dépendance restante :** le dépôt `All-Inclusives/RGAA-BIBLE` est vide ; le workflow et le code n’y ont pas été poussés, aucune paire de clés de publication n’a été créée, et le téléchargement réseau n’est pas encore raccordé.

### Jalon 4 — Interface Tauri

**Objectif :** permettre les parcours essentiels à partir de l’API métier.

- [ ] Confirmer Tauri et sa version, le framework ou JavaScript/TypeScript retenu, et les cibles de publication.
- [ ] Créer les vues de recherche, filtres et résultats.
- [ ] Créer la fiche détaillée et ses états de chargement, erreur et absence de résultat.
- [ ] Relier l’interface au cœur par des commandes Tauri clairement typées.
- [ ] Construire une mise en page responsive, avec navigation clavier et retours visibles.
- [ ] Tester la distribution et l’accès aux données locales sur chaque plateforme prioritaire.

**Livrables :** parcours MVP utilisables, intégration cœur/UI et premiers paquets exécutables.  
**Sortie du jalon :** un utilisateur peut rechercher et ouvrir un critère en ligne comme hors ligne.

### Jalon 5 — Accessibilité, recette et publication

**Objectif :** détecter et corriger les obstacles d’accessibilité et fiabiliser la livraison.

- [ ] Définir une matrice de tests clavier, lecteur d’écran, zoom, contraste et responsive.
- [ ] Tester avec les technologies d’assistance pertinentes pour les plateformes effectivement prises en charge (par exemple NVDA, VoiceOver et TalkBack).
- [ ] Effectuer les tests automatisés disponibles, puis une revue manuelle : l’automatisation seule ne prouve pas la conformité.
- [ ] Vérifier les parcours critiques et les annonces d’état, les titres, le focus et les messages d’erreur.
- [ ] Traiter les anomalies bloquantes et documenter les limites connues.
- [ ] Générer les builds et documenter installation, mise à jour, sources de données et contribution.

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
- Prochaine action recommandée : autoriser ou non l’initialisation/push du projet vers son dépôt GitHub ; le développement local du **jalon 4** peut commencer sans attendre, et c’est là que le premier aperçu visuel sera disponible.

### Journal des jalons

| Date | Jalon | Résultat / décisions |
|---|---|---|
| 4 novembre 2026 | Création du plan | Plan initial établi à partir du cahier des charges ; dépôt DISIC retenu comme source RGAA. |
| 4 novembre 2026 | Inventaire initial | Dépôt cloné à la révision `ca4019f` ; RGAA 4.1.2 courant à la racine (13 thèmes, 106 critères), archive 4.1 séparée ; 119 définitions et 258 méthodologies de test. Licence et limites Web/mobile consignées. |
| 4 novembre 2026 | Cadrage MVP | Périmètre confirmé : corpus officiel RGAA 4.1.2 complet avec enrichissement pédagogique progressif ; Windows desktop comme première cible. |
| 4 novembre 2026 | Jalon 1 terminé | Schéma `1.0.0`, import reproductible et corpus JSON versionné créés. Validation complète : 13 thèmes, 106 critères, 258 tests, 119 définitions ; 258/258 méthodologies correspondantes. Réimport identique ; source inchangée. JSON retenu provisoirement pour le MVP. |
| 4 novembre 2026 | Jalon 2 terminé | Workspace Cargo et crate `a11y-core` créés. API indépendante de Tauri, recherche accent-insensible, filtres et validation de corpus implémentés. 7 tests passent ; Clippy sans avertissement ; benchmark après indexation : ~63,5 µs/recherche en Release. |
| 4 novembre 2026 | Jalon 3 — socle et pipeline local | Manifeste, paquet de test séquence 1, vérification SHA-256, installation versionnée atomique et signature Ed25519 implémentés ; 14 tests passent. Workflow GitHub Releases prêt pour `All-Inclusives/RGAA-BIBLE`, dépôt public actuellement vide. Initialisation/push, clés et téléchargement réseau restent à faire. |
