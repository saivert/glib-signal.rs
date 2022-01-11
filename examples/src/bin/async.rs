use examples::*;
use futures::StreamExt;
use glib::MainLoop;
use glib_signal::ObjectSignalExt;

async fn main_async(mainloop: MainLoop) {
	let obj = TestObject::new();
	let mut stream = obj.clone().signal_stream(TestObjectSomething);

	let arg = "hello";
	obj.something(arg, false);

	let (signal_args,) = stream.next().await.unwrap();
	assert_eq!(signal_args, arg);

	mainloop.quit();
}

fn main() {
	let mainloop = MainLoop::new(None, false);
	let context = mainloop.context();
	context.push_thread_default();

	ctrlc::set_handler({
		let mainloop = mainloop.clone();
		move || mainloop.quit()
	}).unwrap();

	context.spawn_local(main_async(mainloop.clone()));

	mainloop.run();
	context.pop_thread_default();
}
