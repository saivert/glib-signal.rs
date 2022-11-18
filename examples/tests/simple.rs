use {examples::*, glib_signal::ObjectSignalExt};

#[test]
fn simple_signals() {
	let obj = TestObject::new();

	let mut handles = Vec::new();
	for i in 0..2 {
		let handle = obj.handle(TestObjectSomething, move |this, (s,)| {
			println!("TestObject::something({:?}, {:?}) x {}", this, s, i);
			s.len() as u64
		});
		handles.push(handle);
	}
	obj.handle(TestObject::SIGNAL_NOTHING, |this, (s,)| {
		println!("TestObject::nothing({:?}, {:?})", this, s);
	});

	let len = obj.something("whee", false);
	assert_eq!(len, 4 * 2);
	/* TODO: enable once https://github.com/gtk-rs/gtk-rs-core/issues/460 is fixed
	let len = obj.something("whee", true);
	assert_eq!(len, 4*2*2);*/
	for handle in handles {
		obj.remove_handle(handle);
	}
	let len = obj.something("whee", false);
	assert_eq!(len, u64::default());
	obj.nothing("whooo");
}
