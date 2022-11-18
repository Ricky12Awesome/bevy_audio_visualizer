use bevy::{
  prelude::*,
  reflect::TypeUuid,
  render::render_resource::{AsBindGroup, ShaderRef},
  sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
  window::{PresentMode, WindowResized},
};
use bevy_egui::{
  egui,
  egui::{
    plot::{Line, Plot, PlotPoints, PlotUi},
    CollapsingHeader, ComboBox, Ui,
  },
  EguiContext, EguiPlugin, EguiSettings,
};
use safav::{Host, PollingListener};

fn main() {
  App::new()
    .add_plugins(
      DefaultPlugins
        .set(AssetPlugin {
          watch_for_changes: true,
          ..default()
        })
        .set(WindowPlugin {
          window: WindowDescriptor {
            present_mode: PresentMode::AutoNoVsync,
            ..default()
          },
          ..default()
        }),
    )
    .add_plugin(Material2dPlugin::<CustomMaterial>::default())
    .add_plugin(EguiPlugin)
    .insert_resource(EguiSettings {
      scale_factor: 1.0,
      ..default()
    })
    .add_startup_system(setup)
    .add_startup_system(setup_listener)
    .add_system(window_resize)
    .add_system(ui)
    .run();
}

#[derive(Component)]
struct MainCamera;

#[derive(Resource, Deref)]
struct PollingListenerResource(PollingListener);

#[derive(Resource, Deref)]
struct ShaderTestResource(Handle<CustomMaterial>);

fn setup_listener(world: &mut World) {
  let mut host = Host::new().unwrap();
  let listener = PollingListener::default();

  host.listeners().insert("polling", &listener).unwrap();
  host.listen().unwrap();

  // since host isn't "thread safe"
  world.insert_non_send_resource(host);

  // the listener, which is the important one, is thread safe
  world.insert_resource(PollingListenerResource(listener));
}

fn setup(
  mut commands: Commands,
  window: Res<Windows>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut custom_materials: ResMut<Assets<CustomMaterial>>,
) {
  let window = window.primary();
  let surface = shape::Quad::new(Vec2::new(window.width(), window.height()));
  let handle = meshes.set("surface", Mesh::from(surface)).into();
  let material = custom_materials.add(CustomMaterial::default());

  commands.insert_resource(ShaderTestResource(material.clone()));

  commands.spawn_empty().insert(MaterialMesh2dBundle {
    mesh: handle,
    transform: Transform::from_xyz(0.0, 0.5, 0.0),
    visibility: Visibility::VISIBLE,
    material,
    ..default()
  });

  commands.spawn(Camera2dBundle::default()).insert(MainCamera);
}

fn ui(
  mut egui_context: ResMut<EguiContext>,
  handle: Res<ShaderTestResource>,
  mut host: NonSendMut<Host>,
  mut custom_materials: ResMut<Assets<CustomMaterial>>,
  stream: Res<PollingListenerResource>,
) {
  let material = custom_materials.get_mut(&handle.0).unwrap();

  let arr = stream
    .poll()
    .chunks_exact(4)
    .map(Vec4::from_slice)
    .collect::<Vec<_>>();

  material.arr[..arr.len()].copy_from_slice(&arr);
  material.len = arr.len() as u32;

  egui::Window::new("Settings").show(egui_context.ctx_mut(), |ui: &mut Ui| {
    // Slider::new(&mut material.test, 1.0..=10.0)
    //   .text("Size")
    //   .show_value(true)
    //   .logarithmic(true)
    //   .step_by(0.05)
    //   .ui(ui);

    let devices = host.devices();

    let Some(mut selected) = host.current_device_index() else {
      ui.label("Couldn't get any device index");
      return;
    };

    let previous = selected;

    let Some(current) = host.current_device() else {
      return;
    };

    ComboBox::from_label("Select Device")
      .selected_text(current.name())
      .show_ui(ui, |ui: &mut Ui| {
        for (index, dev) in devices.iter().enumerate() {
          ui.selectable_value(&mut selected, index, dev.name());
        }
      });

    if previous != selected {
      host.change_device_by_index(selected).unwrap();
    }

    CollapsingHeader::new("Audio")
      .default_open(false)
      .show(ui, |ui: &mut Ui| {
        let sin = stream
          .poll()
          .iter()
          .enumerate()
          .map(|(index, value): (usize, &f32)| [index as f64, *value as f64])
          .collect::<PlotPoints>();

        let line = Line::new(sin);

        Plot::new("my_plot")
          .allow_drag(false)
          .allow_scroll(false)
          .allow_zoom(false)
          .allow_boxed_zoom(false)
          .center_y_axis(true)
          .include_y(0.75)
          .include_x(500.0)
          .show(ui, |plot_ui: &mut PlotUi| plot_ui.line(line))
      });
  });
}

fn window_resize(mut meshes: ResMut<Assets<Mesh>>, mut events: EventReader<WindowResized>) {
  for event in events.iter() {
    let size = Vec2::new(event.width, event.height);
    let surface = shape::Quad::new(size);
    let _ = meshes.set("surface", Mesh::from(surface));
  }
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "b62bb455-a72c-4b56-87bb-81e0554e234f"]
pub struct CustomMaterial {
  #[uniform(0)]
  pub arr: Box<[Vec4; 4096]>,
  #[uniform(1)]
  pub len: u32,
  // #[texture(0)]
  // #[sampler(1)]
  // texture: Handle<Image>,
}

impl Default for CustomMaterial {
  fn default() -> Self {
    Self {
      arr: Box::new([Vec4::ZERO; 4096]),
      len: 4,
    }
  }
}

impl Material2d for CustomMaterial {
  fn fragment_shader() -> ShaderRef {
    "shaders/shader.wgsl".into()
  }
}
