//! A Windows WM_COPYDATA callback must return before doing anything that can
//! acquire Tauri's window locks or marshal work back to the UI thread.
use std::sync::mpsc::{self, SendError, Sender};

pub struct LaunchRequest {
    pub args: Vec<String>,
    pub cwd: String,
}

pub struct LaunchDispatcher(Sender<LaunchRequest>);

impl LaunchDispatcher {
    pub fn new(mut handle: impl FnMut(LaunchRequest) + Send + 'static) -> Self {
        let (sender, receiver) = mpsc::channel();
        std::thread::Builder::new()
            .name("pdf-launch-requests".into())
            .spawn(move || {
                for request in receiver {
                    handle(request);
                }
            })
            .expect("Could not start PDF launch worker");
        Self(sender)
    }

    pub fn send(&self, args: Vec<String>, cwd: String) -> Result<(), SendError<LaunchRequest>> {
        self.0.send(LaunchRequest { args, cwd })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn launch_callback_can_return_while_window_creation_is_blocked() {
        let (started, wait_started) = mpsc::channel();
        let (resume, wait_resume) = mpsc::channel();
        let (finished, wait_finished) = mpsc::channel();
        let dispatcher = LaunchDispatcher::new(move |request| {
            started.send(()).unwrap();
            wait_resume.recv().unwrap();
            finished.send(request.args).unwrap();
        });
        dispatcher
            .send(vec!["first.pdf".into()], "C:\\資料".into())
            .unwrap();
        wait_started.recv_timeout(Duration::from_secs(2)).unwrap();
        // A second launch must not wait for window creation or the first request.
        dispatcher
            .send(vec!["second.pdf".into()], "C:\\資料".into())
            .unwrap();
        resume.send(()).unwrap();
        assert_eq!(
            wait_finished.recv_timeout(Duration::from_secs(2)).unwrap(),
            ["first.pdf"]
        );
        wait_started.recv_timeout(Duration::from_secs(2)).unwrap();
        resume.send(()).unwrap();
        assert_eq!(
            wait_finished.recv_timeout(Duration::from_secs(2)).unwrap(),
            ["second.pdf"]
        );
    }

    #[test]
    fn burst_requests_keep_their_order_arguments_and_sender_directories() {
        let (done, received) = mpsc::channel();
        let dispatcher = LaunchDispatcher::new(move |request| {
            done.send(request).unwrap();
        });
        for index in 0..20 {
            dispatcher
                .send(
                    vec!["komodo.exe".into(), format!("日本語 #{index}.pdf")],
                    format!("C:\\sender{index}"),
                )
                .unwrap();
        }
        for index in 0..20 {
            let request = received.recv_timeout(Duration::from_secs(2)).unwrap();
            assert_eq!(request.args[1], format!("日本語 #{index}.pdf"));
            assert_eq!(request.cwd, format!("C:\\sender{index}"));
        }
    }
}
