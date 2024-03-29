extern crate noise;
use bevy::{input::mouse::MouseWheel, prelude::*};
use noise::{NoiseFn, Perlin, Seedable};
use rand::Rng;
use std::f32::consts::PI;

const ENERGIE_SPRITE: &str = "textures/energie.png";
const MINERAL_SPRITE: &str = "textures/minerai.png";
const LIEU_INTERET_SPRITE: &str = "textures/lieu.png";
const BASE_SPRITE: &str = "textures/base.png";
const ROBOT_SPRITE: &str = "textures/robot.png";
const OBSTACLE_SPRITE: &str = "textures/obstacle.png";

#[derive(Component, PartialEq, Debug)]
enum Ressource {
    Energie,
    Mineral,
    LieuInteretScientifique,
}

#[derive(Component, Debug)]
struct Carte {
    largeur: usize,
    hauteur: usize,
}

#[derive(Component, PartialEq, Debug)]
enum TypeRobot {
    Explorateur,
    Collecteur,
    Visiteur,
}

#[derive(Component, Debug)]
struct Robot {
    id: i32,
    nom: String,
    pv_max: i32,
    type_robot: TypeRobot,
    vitesse: i32,
}

#[derive(Component, PartialEq, Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component, Debug)]
struct Obstacle {
    id: i32,
}

#[derive(Component, Debug)]
struct Bordure;

#[derive(Component, Debug)]
struct Base;

/***
 * Fonction pour la caméra
 */
fn setup_camera(mut commands: Commands) {
    let zoom_level = 0.05; 
    let map_center_x = 10.0; 
    let map_center_y = 10.0; 

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(map_center_x, map_center_y, 10.0)
                   .with_scale(Vec3::new(zoom_level, zoom_level, 1.0)),
        ..default()
    });
    commands.insert_resource(ClearColor(Color::rgb(0.5, 0.5, 0.5))); // Définit la couleur de fond à gris    
}

/***
 * Fonction pour charger la map
 */
fn setup_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Charger les textures pour les différents éléments de la carte
    let energie_texture_handle = asset_server.load(ENERGIE_SPRITE);
    let mineral_texture_handle = asset_server.load(MINERAL_SPRITE);
    let lieu_interet_texture_handle = asset_server.load(LIEU_INTERET_SPRITE);
    let base_handle = asset_server.load(BASE_SPRITE);
    let obstacle_handle = asset_server.load(OBSTACLE_SPRITE); // Assurez-vous que c'est le bon chemin de fichier

    // Définir les dimensions de la carte
    let largeur = 50;
    let hauteur = 50;

    // Créer l'entité de la carte avec sa position de base
    commands.spawn((Carte { largeur, hauteur }, Position { x: 0, y: 0 }));

    let seed = rand::thread_rng().gen();
    let perlin = Perlin::new(seed);

    // Génération des éléments de la carte en fonction de la valeur du noise
    for y in 0..hauteur {
        for x in 0..largeur {
            let position = Position { x: x as i32, y: y as i32 };
            let noise_value = perlin.get([x as f64 * 0.1, y as f64 * 0.1]);
            let noise_normalised = (noise_value + 1.0) / 2.0;

            // Générer un obstacle si la valeur du bruit est supérieure à 0.6
            if noise_normalised > 0.8 {
                commands.spawn(SpriteBundle {
                    texture: obstacle_handle.clone(),
                    transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 0.0))
                                   .with_scale(Vec3::splat(0.00038)),
                        ..Default::default()
                }).insert(Obstacle { id: rand::thread_rng().gen() })
                .insert(position);
            } else {
                // Déterminer quel type de ressource générer en fonction de la valeur du bruit
                let sprite = match noise_normalised {
                    n if n > 0.75 => Some((Ressource::Energie, energie_texture_handle.clone())),
                    n if n > 0.72 => Some((Ressource::Mineral, mineral_texture_handle.clone())),
                    n if n > 0.7 => Some((Ressource::LieuInteretScientifique, lieu_interet_texture_handle.clone())),
                    _ => None,
                };

                if let Some((ressource, texture_handle)) = sprite {
                    commands.spawn(SpriteBundle {
                        texture: texture_handle,
                        transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 0.0))
                                   .with_scale(Vec3::splat(0.00038)),
                        ..Default::default()
                    })
                    .insert(ressource)
                    .insert(position);
                }
            }
        }
    }

    // Ajout de la base sur la carte
    let base_x = rand::thread_rng().gen_range(0..largeur) as i32;
    let base_y = rand::thread_rng().gen_range(0..hauteur) as i32;
    commands.spawn(SpriteBundle {
        texture: base_handle,
        transform: Transform::from_translation(Vec3::new(base_x as f32, base_y as f32, 0.0))
                   .with_scale(Vec3::splat(0.002)),
        ..Default::default()
    })
    .insert(Base)
    .insert(Position { x: base_x, y: base_y }); 
}

/***
 * Fonction pour ajouter les bordures
 */
fn setup_bordures(
    mut commands: Commands,
    query: Query<(&Carte, &Position)>,
) {
    for (carte, carte_position) in query.iter() {
        let bordure_couleur = Color::BLACK; 
        let epaisseur_bordure = 0.05; 
        let taille_case = 1.0; 
        println!("{}", carte.hauteur);
        for y in 0..carte.hauteur {
            for x in 0..carte.largeur {
                let x_pos = x as f32 + carte_position.x as f32 * taille_case;
                let y_pos = y as f32 + carte_position.y as f32 * taille_case;

                // Créer les bordures verticales
                if x < carte.largeur - 1 {
                    commands.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: bordure_couleur,
                            custom_size: Some(Vec2::new(epaisseur_bordure, taille_case)),
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(x_pos + 0.5 * taille_case, y_pos, 2.0), 
                        ..Default::default()
                    });
                }

                // Créer les bordures horizontales
                if y < carte.hauteur - 1 {
                    commands.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: bordure_couleur,
                            custom_size: Some(Vec2::new(taille_case, epaisseur_bordure)),
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(x_pos, y_pos + 0.5 * taille_case, 2.0), // Ajustez le Z pour s'assurer qu'il est visible
                        ..Default::default()
                    });
                }
            }
        }
    }
}

/***
 * Fonction d'ajout des robots sur la carte
 */
fn spawn_robots(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(&Carte, &Position)>
) {
    let robot_texture_handle = asset_server.load(ROBOT_SPRITE);

    if let Some((carte, _)) = query.iter().next() {
        for id in 1..=5 {
            let robot_x: i32 = rand::thread_rng().gen_range(0..carte.largeur) as i32;
            let robot_y: i32 = rand::thread_rng().gen_range(0..carte.hauteur) as i32;

            let type_robot = match id % 3 {
                0 => TypeRobot::Explorateur,
                1 => TypeRobot::Collecteur,
                _ => TypeRobot::Visiteur,
            };

            let robot_name = match type_robot {
                TypeRobot::Explorateur => format!("Explorateur{}", id),
                TypeRobot::Collecteur => format!("Collecteur{}", id),
                TypeRobot::Visiteur => format!("Visiteur{}", id),
            };

            commands.spawn(SpriteBundle {
                texture: robot_texture_handle.clone(),
                transform: Transform::from_translation(Vec3::new(robot_x as f32, robot_y as f32, 1.0))
                           .with_scale(Vec3::splat(0.003)),
                ..Default::default()
            }).insert(Robot {
                id: id,
                nom: robot_name,
                pv_max: 100,
                type_robot: type_robot,
                vitesse: 1
            }).insert(Position { x: robot_x, y: robot_y });
        }
    }
}

/***
 * Fonction de collecte des ressources si un robot est sur la même position qu'une ressource
 */
fn collect_resources_system(
    mut commands: Commands,
    mut robot_query: Query<(Entity, &mut Robot, &Position)>,
    resource_query: Query<(Entity, &Ressource, &Position)>,
) {
    println!("Resources available: {}", resource_query.iter().count()); 
    for (robot_entity, mut robot, robot_position) in robot_query.iter_mut() {
        println!("{:?}", robot.type_robot);
        if robot.type_robot == TypeRobot::Collecteur {
        println!("Checking robot {} at position {:?}", robot.nom, robot_position); 
        let mut resource_collected = false; 
        for (resource_entity, resource, resource_position) in resource_query.iter() {
            if robot_position == resource_position {
                resource_collected = true; // La ressource a été trouvée à la même position que le robot
                match resource {
                    Ressource::Energie => {
                        println!("Robot {} collected energy at position {:?}", robot.nom, robot_position);
                    },
                    Ressource::Mineral => {
                        println!("Robot {} collected mineral at position {:?}", robot.nom, robot_position); 
                    },
                    Ressource::LieuInteretScientifique => {
                        println!("Robot {} discovered a place of interest at position {:?}", robot.nom, robot_position); 
                    },
                }
                commands.entity(resource_entity).despawn();
            }
        }
        if !resource_collected {
            println!("Robot {} did not collect any resources at position {:?}", robot.nom, robot_position); 
        }
        }
    }
}

/***
 * Fonction pour déplacer la caméra avec les touches directionnelles
 */
fn move_camera_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera2d>>,
) {
    let camera_speed = 10.0;

    for mut transform in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            transform.translation.x -= camera_speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            transform.translation.x += camera_speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            transform.translation.y += camera_speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            transform.translation.y -= camera_speed * time.delta_seconds();
        }
    }
}

/***
 * Fonction pour faire un zoom avec la caméra
 */
fn zoom_camera_system(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<Camera2d>>,
) {
    let mut zoom_change = 0.0;
    for event in mouse_wheel_events.read() {
        zoom_change += event.y * 0.01; 
    }

    if zoom_change != 0.0 {
        for mut transform in query.iter_mut() {
            transform.scale *= Vec3::new(1.0 + zoom_change, 1.0 + zoom_change, 1.0);
            transform.scale = transform.scale.clamp(Vec3::splat(0.03), Vec3::splat(5.0));
        }
    }
}

/***
 * Fonction pour déplacer les robots
 */
fn move_robots_on_map_system(
    mut query: Query<(Entity, &mut Position, &mut Transform, &Robot)>,
    carte_query: Query<&Carte>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    if timer.duration().as_secs_f32() == 0.0 {
        *timer = Timer::from_seconds(1.0, TimerMode::Repeating);
    }

// On attend que le timer se finit avant de déplacer le robot
    if !timer.tick(time.delta()).just_finished() {
        return;
    }

    let carte = carte_query.single(); 

    for (entity, mut position, mut transform, _) in query.iter_mut() {
        // Si le robot est dans la base, le laisser pour l'instant
        if position.x == (carte.largeur / 2) as i32 && position.y == (carte.hauteur / 2) as i32 {
            continue; 
        }
        
        // Avancer le robot d'une case
        position.x = (position.x + 1) % carte.largeur as i32;
        position.y = (position.y + 1) % carte.hauteur as i32;
        
        transform.translation.x = position.x as f32;
        transform.translation.y = position.y as f32;
    }
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
              title: "Essaim de Robots pour Exploration et Etude Astrobiologique".to_string(),
              ..default()
            }),
            ..default()
          }))
        .insert_resource(ClearColor(Color::rgb(0.5, 0.5, 0.5)))
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup_map)
        .add_systems(PostStartup, setup_bordures)
        .add_systems(PostStartup, spawn_robots)
        .add_systems(Update, move_robots_on_map_system)
        .add_systems(Update, collect_resources_system)
        .add_systems(Update, move_camera_system)
        .add_systems(Update, zoom_camera_system)
        .run();
}
