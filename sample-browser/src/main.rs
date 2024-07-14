//use dataset_tools::walk_directory;
use egui::{ CentralPanel, ScrollArea, Ui };
use std::path::{ Path, PathBuf };
use std::sync::{ Arc, Mutex };
use rodio::{ OutputStream, OutputStreamHandle, Sink, Decoder };
use std::fs::File;
use std::io::BufReader;
use walkdir::WalkDir;

#[derive(Clone)]
struct AudioPlayer {
    audio_files: Arc<Mutex<Vec<PathBuf>>>,
    current_playing: Arc<Mutex<Option<PathBuf>>>,
    sink: Arc<Mutex<Option<Sink>>>,
    _stream: Arc<OutputStream>,
    stream_handle: Arc<OutputStreamHandle>,
}

impl AudioPlayer {
    fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        Self {
            audio_files: Arc::new(Mutex::new(Vec::new())),
            current_playing: Arc::new(Mutex::new(None)),
            sink: Arc::new(Mutex::new(None)),
            _stream: Arc::new(stream),
            stream_handle: Arc::new(stream_handle),
        }
    }

    async fn load_audio_files(&self) -> anyhow::Result<()> {
        let audio_dir = Path::new("E:\\Audio");
        let audio_files = self.audio_files.clone();

        for entry in WalkDir::new(audio_dir)
            .into_iter()
            .filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension.eq_ignore_ascii_case("wav") || extension.eq_ignore_ascii_case("ogg") {
                    audio_files.lock().unwrap().push(path.to_path_buf());
                }
            }
        }

        self.audio_files.lock().unwrap().sort();
        Ok(())
    }

    fn play_audio(&self, path: &Path) {
        if let Ok(file) = File::open(path) {
            if let Ok(source) = Decoder::new(BufReader::new(file)) {
                let sink = Sink::try_new(&self.stream_handle).unwrap();
                sink.append(source);
                sink.play();
                *self.sink.lock().unwrap() = Some(sink);
                *self.current_playing.lock().unwrap() = Some(path.to_path_buf());
            }
        }
    }

    fn stop_audio(&self) {
        if let Some(sink) = self.sink.lock().unwrap().take() {
            sink.stop();
        }
        *self.current_playing.lock().unwrap() = None;
    }
}

fn main() -> Result<(), eframe::Error> {
    let audio_player = AudioPlayer::new();

    // Load audio files asynchronously
    let audio_player_clone = audio_player.clone();
    tokio::runtime::Runtime
        ::new()
        .unwrap()
        .block_on(async move {
            if let Err(e) = audio_player_clone.load_audio_files().await {
                eprintln!("Error loading audio files: {}", e);
            }
        });

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Sample Browser",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::new(audio_player))))
    )
}

struct MyApp {
    audio_player: AudioPlayer,
}

impl MyApp {
    fn new(audio_player: AudioPlayer) -> Self {
        Self { audio_player }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Sample Browser");
            ScrollArea::vertical().show(ui, |ui| {
                self.audio_list(ui);
            });
        });
    }
}

impl MyApp {
    fn audio_list(&self, ui: &mut Ui) {
        for path in self.audio_player.audio_files.lock().unwrap().iter() {
            let file_name = path.file_name().unwrap().to_string_lossy();
            if ui.button(file_name.to_string()).clicked() {
                if Some(path.to_path_buf()) == *self.audio_player.current_playing.lock().unwrap() {
                    self.audio_player.stop_audio();
                } else {
                    self.audio_player.play_audio(path);
                }
            }
        }
    }
}
