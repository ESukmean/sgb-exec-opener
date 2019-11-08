use super::*;
use std::io::Cursor;
use std::path::Path;
use tar::Archive;
use std::process::Command;
use std::thread;
use std::time;

pub fn new(tx: std::sync::mpsc::Sender<Progress>) -> Loader{
	Loader{
		tx: tx
	}
}
pub struct Loader{
	tx: std::sync::mpsc::Sender<Progress>
}

impl Loader{
	pub fn run(&mut self){
		let mut total_step = ProgressStep::new(0, 4);

		match self.tx.send(Progress::new("초기화 하는중...".to_string(), total_step, ProgressStep::new(2, 10))){
			_ => ()
		}

		if Path::new("/tmp/firefox").exists() {
			total_step.step_up();
			match self.tx.send(Progress::new("파일이 이미 존재합니다".to_string(), total_step, ProgressStep::new(0, 1))){
				_ => ()
			};
		} else {
			total_step.step_up();
			match self.tx.send(Progress::new("압축 푸는중...".to_string(), total_step, ProgressStep::new(0, -1))){
				_ => ()
			}
			
			{
				let targz = include_bytes!("firefox.tar.gz");
				let targz_buf = Cursor::new(&targz[..]);

				let tar = flate2::read::GzDecoder::new(targz_buf);
				let mut archive = Archive::new(tar);

				total_step.step_up();
				match archive.unpack("/tmp/"){
					Err(_) => {
						match self.tx.send(Progress::new("압축 해제중 오류발생".to_string(), total_step, ProgressStep::new(2, 10))){
							_ => ()
						};
						return;
					},
					Ok(_) => ()
				};
			}
		}

		total_step.step_up();
		match self.tx.send(Progress::new("실행중...".to_string(), total_step, ProgressStep::new(2, 10))){
			_ => ()
		};

		total_step.step_up();

		match Command::new("/tmp/firefox/firefox/firefox").spawn(){
			Ok(_) => match self.tx.send(Progress::new("완료".to_string(), total_step, ProgressStep::new(1, 1))){
				_ => ()
			},
			Err(e) => match self.tx.send(Progress::new(format!("실행중 오류발생: {}", e), ProgressStep::new(1, 1), ProgressStep::new(1, 1))){
				_ => ()
			},
		};

		let wait = time::Duration::from_millis(1000 * 3);
		thread::sleep(wait);

		match self.tx.send(Progress::new("종료".to_string(), total_step, ProgressStep::new(0, 0))){
			_ => ()
		};
	}
}

