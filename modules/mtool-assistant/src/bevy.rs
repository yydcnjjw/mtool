use bevy::{
    ecs::schedule::ScheduleLabel,
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
    render::camera::RenderTarget,
    text::FontSmoothing,
    ui::experimental::UiRootNodes,
    window::WindowRef,
};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiContextPass, EguiMultipassSchedule, EguiPlugin},
    bevy_inspector, egui,
    quick::WorldInspectorPlugin,
    DefaultInspectorConfigPlugin,
};
use mapp::{anyhow, provider::Res as AppRes};
use smooth_bevy_cameras::{controllers::unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin}, LookTransformPlugin};
use std::{f32::consts::*, ops::Deref, time::Duration};

use crate::context::BevyState;

fn is_window_ready(state: Res<BevyState>) -> bool {
    state.window_id().is_some()
}

pub fn assistant(app: &mut App, state: AppRes<BevyState>) -> Result<(), anyhow::Error> {
    app.insert_resource(state.deref().to_owned())
        .add_systems(
            Update,
            (
                wait_for_window,
                animate_light_direction.run_if(is_window_ready),
            ),
        )
        
        // .add_systems(WindowContextPass, inspector_ui.run_if(is_window_ready))
        ;

    use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
    app.add_plugins(FpsOverlayPlugin {
        config: FpsOverlayConfig {
            text_config: TextFont {
                // Here we define size of our overlay
                font_size: 42.0,
                // If we want, we can use a custom font
                font: default(),
                // We could also disable font smoothing,
                font_smoothing: FontSmoothing::default(),
                ..default()
            },
            // We can also change color of the overlay
            text_color: Color::WHITE,
            // We can also set the refresh interval for the FPS counter
            refresh_interval: Duration::from_millis(100),
            enabled: true,
        },
    })
        .add_plugins(LookTransformPlugin)
        .add_plugins(UnrealCameraPlugin::default());

    // .add_plugins(EguiPlugin {
    //     enable_multipass_for_primary_context: true,
    // });
    // .add_plugins(DefaultInspectorConfigPlugin);
    Ok(())
}

fn inspector_ui(world: &mut World) {
    let state = world.resource::<BevyState>();
    let mut egui_context =
        if let Some(ctx) = world.entity(state.window_id_uncheck()).get::<EguiContext>() {
            ctx.clone()
        } else {
            return;
        };

    egui::Window::new("World Inspector")
        .default_size((320., 160.))
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                bevy_inspector::ui_for_world(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WindowContextPass;

fn setup(In(window_id): In<Entity>, mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("setup window: {}", window_id);

    commands
        .entity(window_id)
        .insert(EguiMultipassSchedule::new(WindowContextPass));

    commands.spawn((
        Camera3d::default(),
        // Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 250.0,
            ..default()
        },
        Camera {
            target: RenderTarget::Window(WindowRef::Entity(window_id)),
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        IsDefaultUiCamera,
        UnrealCameraBundle::new(
            UnrealCameraController::default(),
            Vec3::new(-2.0, 5.0, 5.0),
            Vec3::new(0., 0., 0.),
            Vec3::Y,
        )
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .build(),
    ));
    commands.spawn((SceneRoot(asset_server.load("test.glb#Scene0")),));
}

fn wait_for_window(mut commands: Commands, state: Res<BevyState>, mut window_created: Local<bool>) {
    if *window_created {
        return;
    }
    if let Some(id) = state.window_id() {
        *window_created = true;
        let setup_id = commands.register_system(setup);
        commands.run_system_with(setup_id, id);
    }
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_secs() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}
