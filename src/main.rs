#[macro_use] extern crate lazy_static;
extern crate iui;
extern crate flate2;
extern crate tar;

use iui::prelude::*;
use iui::controls::*;

use std::thread;
use std::sync::mpsc::channel;

mod loading;

#[derive(Copy, Clone, Debug)]
pub struct ProgressStep{
	step: i64,
	total: i64
}
#[derive(Clone, Debug)]
pub struct Progress{
	help: String,

	overall_step: ProgressStep,
	subset_step: ProgressStep,
}

impl ProgressStep{
	pub fn new(progress: i64, total: i64) -> Self{
		return ProgressStep{
			step: progress,
			total: total
		};
	}

	pub fn step_up(&mut self){
		self.step += 1;
	}
	pub fn set_step(&mut self, step: i64){
		self.step = step;
	}
	pub fn set_total(&mut self, total: i64){
		self.total = total;
	}
}
impl Progress{
	pub fn new(help: String, overall_step: ProgressStep, subset_step: ProgressStep) -> Self{
		return Progress{
			help: help,
			
			overall_step: overall_step,
			subset_step: subset_step
		};
	}
}

fn main() {
	let ui = UI::init().expect("UI 생성 실패");
	let mut win = Window::new(&ui, "실행 준비중...", 300, 100, WindowType::NoMenubar);
	
	let (tx, rx) = channel::<Progress>();
	thread::spawn(|| loading::new(tx).run());

	let mut vbox = VerticalBox::new(&ui);
	vbox.set_padded(&ui, true);

	let label = Label::new(&ui, "초기화 중...");
	let progress_bar = ProgressBar::new();


	let mut event_loop = ui.event_loop();

	event_loop.on_tick(&ui, {
		let ui = ui.clone();
		let progress_bar = progress_bar.clone();
		let mut label = label.clone();
		lazy_static!{
			static ref SIGNAL_QUIT: String = "종료".to_string();
		};
		move || {
			if let Ok(r) = rx.try_recv(){
				if r.help == *SIGNAL_QUIT{
					ui.quit();
				}

				let overall = r.overall_step.step as f32 / r.overall_step.total as f32;
				let subset = (r.subset_step.step as f32 / r.subset_step.total as f32) / r.overall_step.total as f32;

				let progress = ((overall + subset) * 100f32) as u32;

				label.set_text(&ui, r.help.as_str());

				if r.subset_step.total == -1 {
					progress_bar.clone().set_value(&ui, None);
				}else{
					progress_bar.clone().set_value(&ui, progress);
				}
			}
		}
	});

	vbox.append(&ui, label, LayoutStrategy::Compact);
	vbox.append(&ui, progress_bar, LayoutStrategy::Stretchy);

	win.set_child(&ui, vbox);
	win.show(&ui);

	event_loop.run(&ui);
}