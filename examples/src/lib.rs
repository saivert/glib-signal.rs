use {
	glib::ToValue,
	glib_signal::{BuildSignal, BuildableSignal, DetailedSignal, Signal, SignalFlags},
};

mod imp {
	use {
		glib::{
			once_cell::sync::OnceCell,
			subclass::{object::ObjectImpl, signal::Signal, types::ObjectSubclass},
		},
		glib_signal::BuildSignal as _,
	};

	#[derive(Default)]
	pub struct TestObject {}

	#[glib::object_subclass]
	impl ObjectSubclass for TestObject {
		type Interfaces = ();
		type ParentType = glib::Object;
		type Type = super::TestObject;

		const NAME: &'static str = "TestObject";
	}

	impl ObjectImpl for TestObject {
		fn signals() -> &'static [Signal] {
			static SIGNALS: OnceCell<[Signal; 2]> = OnceCell::new();
			SIGNALS.get_or_init(|| [super::TestObjectSomething::build(), super::TestObjectNothing::build()])
		}
	}
}

glib::wrapper! {
	pub struct TestObject(ObjectSubclass<imp::TestObject>);
}

impl TestObject {
	pub fn new() -> Self {
		glib::Object::new()
	}

	pub fn something(&self, s: &str, else_: bool) -> u64 {
		use glib::ObjectExt;
		let s = s.to_value();
		let something = TestObjectSomething::signal();
		if else_ {
			self.emit_with_details(something, TestObjectSomethingElse::create_detail(), &[&s])
		} else {
			self.emit(something, &[&s])
		}
	}

	pub fn nothing(&self, s: &str) {
		use glib::ObjectExt;
		let s = s.to_value();
		let nothing = TestObjectNothing::signal();
		self.emit(nothing, &[&s])
	}
}

glib_signal::def_signal! {
	impl Notifies<"nothing" as TestObjectNothing> for TestObject {
		impl {const SIGNAL_NOTHING};
		impl BuildSignal;
		fn(&self, String)
	}
}

glib_signal::def_signal! {
	impl Notifies<"something" as TestObjectSomething> for TestObject {
		impl {const SIGNAL_SOMETHING};
		FLAGS = SignalFlags::DETAILED;
		fn(&self, String) -> u64
	}
}

#[derive(Copy, Clone, Debug)]
pub struct TestObjectSomethingElse;
impl DetailedSignal for TestObjectSomethingElse {
	type Arguments = <TestObjectSomething as Signal>::Arguments;
	type Object = <TestObjectSomething as Signal>::Object;
	type Return = <TestObjectSomething as Signal>::Return;
	type Signal = TestObjectSomething;

	const DETAIL: Option<&'static str> = Some("else");
}

impl BuildSignal for TestObjectSomething {
	fn build() -> glib::subclass::Signal {
		Self::builder(|b| {
			b.accumulator(|cx, lhs, rhs| {
				if cx.detail() == Some(TestObjectSomethingElse::create_detail()) {
					*lhs = (lhs.get::<u64>().unwrap() + rhs.get::<u64>().unwrap() * 2).to_value();
				} else {
					*lhs = (lhs.get::<u64>().unwrap() + rhs.get::<u64>().unwrap()).to_value();
				}
				true
			})
			.build()
		})
	}
}
