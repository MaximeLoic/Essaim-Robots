# Essaim-Robots

## Fonctionnalités principales

- **Génération procédurale** d’une carte à l’aide de bruit de Perlin
- **Exploration autonome** par les robots
- **Détection et collecte de ressources** (via comportements programmés)
- **Gestion des collisions** avec obstacles et limites de la carte
- **Caméra libre** avec déplacement (bouton de direction)
- **Interface utilisateur (UI)** affichant le score en temps réel et faisant disparaître les ressources collectées
- **Architecture modulaire** basée sur ECS avec séparation logique et scalable du comportement des entités

## Lancement de l'application

1. Télécharger le projet
```bash
git clone https://github.com/MaximeLoic/Essaim-Robots
```
2. Entrer dans le dossier du projet
```bash
cd Essaim-Robots    
```
3. Lancer la commande 
```bash
cargo run
```

## Exemple de capture d'écran



## ADR : Architecture Decision Record

### Submitters
- Paul-Henry NGANKAM NGOUNOU
- Thierry Pavone TCHOUAMOU PAYONG
- Maxime Loïc NKWEMI NJIKI
- Oumou Khairy GUEYE

### Change Log

2025-04-06 : Création et enrichissement basé sur [les commits](https://github.com/MaximeLoic/Essaim-Robots/commits/main/)

### Cas d'utilisation référencé

Simulation interactive de robots explorant un environnement 2D, détectant des ressources, les collectant, évitant des obstacles, et mettant à jour une UI en temps réel avec le score et les événements.

### Contexte

Le simulateur a été initié comme une preuve de concept dans un unique fichier main.rs, combinant la logique de la carte, des robots, et de l’UI. À mesure que la complexité a augmenté — ajout de la fonction noise pour la génération de carte et des obstacles, gestion de caméra, mouvement de robots, détection/collision, collecte de ressources, etc. — la nécessité d’une séparation claire des responsabilités s’est imposée.

#### Points de complexité rencontrés

- Génération procédurale de la carte à l’aide de la fonction noise

- Affichage d’une grande map via une caméra mobile

- Gestion de multiples robots avec mouvements, détection et collecte

- Mise à jour visuelle en temps réel des scores et interactions

- Collision et comportements d’évitement / interaction avec obstacles

### Design proposé

#### Découpage en modules Rust
Le code a été refactoré en 6 fichiers principaux :

- main.rs : Point d’entrée de l’application, initialise les système.

- map.rs : Génération de la carte avec bruit de Perlin/Simplex, gestion des tuiles, rendu.

- robot.rs : Logique des robots (spawn, mouvement, détection, collecte), comportements d’exploration.

- ui.rs : Affichage de l’interface, score des robots, informations en temps réel.

- common.rs : Composants et structures partagées (ex : types, constantes, systèmes de base).

- lib.rs : Coordination entre les modules pour exposer les systèmes à main.rs.

#### Architecture fonctionnelle

- Carte : générée gâce à la fonction noise, chaque tuile pouvant être vide, ou composer une ressource ou un obstacle.

- Robots : entités autonomes qui explorent la carte. Munis de comportements primaire (patrouille, détection de ressources à proximité, évitement d’obstacles).

- Systèmes ECS : les fonctionnalités sont découpées en systèmes (spawn, input, mouvement, collecte, collision).

- UI : score affiché en temps réel, mise à jour via un système dédié connecté aux événements de collecte.

- Utilisation de la bibliothèque Bevy : moteur de jeu Rust moderne basé sur ECS, offrant un pipeline de rendu performant, un système de plugins, une compatibilité multiplateforme, et une intégration fluide pour le développement modulaire et data-driven.

#### Design patterns

- Usage du pattern ECS (Entity-Component-System) pour une séparation logique et scalable du comportement des entités.

- Architecture orientée data-driven, ce qui rend les règles de comportement facilement modifiables.

- Passage d’un code centralisé à un design modulaire, pour faciliter la maintenance et l’évolution du projet.

### Considerations

#### Avantages

- Architecture modulaire facilement extensible (ajout d’autres entités, règles, capteurs).

- Code plus lisible.

- Meilleure séparation des préoccupations (carte, logique robot, UI).

#### Inconvénients

- Augmentation de la complexité initiale du projet.

- Coordination entre systèmes distribués dans plusieurs modules nécessite rigueur.

#### Alternatives envisagés

- Garder un seul fichier main.rs : vite ingérable.

### Decision

#### Adoption d’une architecture modulaire Rust avec séparation par responsabilités métier :

- Carte

- Robots

- UI

- Logique partagée

Ce découpage améliore la lisibilité, la réutilisabilité et l’évolutivité du code. Il s’appuie sur une logique ECS qui permet d’itérer rapidement tout en gardant les bases solides pour une simulation plus avancée par la suite (gestion d’équipe, scoring complexe...).

### Références

- [Bevy](https://bevyengine.org/)
- [ECS](https://ianjk.com/ecs-in-rust/)
- [Noise](https://github.com/Razaekel/noise-rs)
- [ADR template](https://docs.edgexfoundry.org/2.3/design/adr/template/)
