#![doc(html_root_url = "https://docs.rs/glib-signal/0.2.0/")]
#![cfg_attr(feature = "dox", feature(doc_notable_trait, doc_cfg))]

#[cfg(feature = "futures")]
pub use self::signal_stream::{ConnectEof, OnceFuture, SignalStream};
#[doc(hidden)]
pub use glib; // for macro use
pub use {
	self::{
		borrowed_object::BorrowedObject,
		from_values::FromValues,
		pointer::Pointer,
		value_option::{PrimitiveValue, ToValueOption},
	},
	glib::SignalFlags,
};
use {
	glib::{
		subclass::{signal::SignalBuilder, SignalId},
		translate::{from_glib, IntoGlib, ToGlibPtr},
		value::FromValue,
		BoolError, Closure, ObjectExt, ObjectType, Quark, SignalHandlerId, StaticType,
	},
	std::{fmt::Debug, marker::PhantomData},
};

#[cfg(feature = "futures")]
mod signal_stream;

mod borrowed_object;

mod pointer;

mod value_option;

mod from_values;

mod macros;

pub trait Signal: Copy + Debug {
	type Object: ObjectType;
	type Arguments: for<'a> FromValues<'a> + 'static;
	type Return: ToValueOption;

	const NAME: &'static str;
	const FLAGS: SignalFlags = SignalFlags::empty();

	fn signal() -> SignalId {
		SignalId::lookup(Self::NAME, <Self::Object as StaticType>::static_type()).expect(Self::NAME)
	}
}

pub trait DetailedSignal: Copy + Debug + Into<ConnectDetails<Self>> {
	type Signal: Signal;
	type Object: ObjectType;
	type Arguments: for<'a> FromValues<'a> + 'static;
	type Return: ToValueOption;

	const DETAIL: Option<&'static str>;

	fn detail() -> Option<Quark> {
		match Self::DETAIL {
			Some(detail) => Some(Quark::try_from_str(detail).expect(detail)),
			None => None,
		}
	}

	fn create_detail() -> Quark {
		Quark::from_str(Self::DETAIL.expect("detail string required"))
	}
}

impl<T: Signal> DetailedSignal for T {
	type Arguments = <Self::Signal as Signal>::Arguments;
	type Object = <Self::Signal as Signal>::Object;
	type Return = <Self::Signal as Signal>::Return;
	type Signal = Self;

	const DETAIL: Option<&'static str> = None;
}

pub trait BuildableSignal: Signal {
	fn builder<F: FnOnce(SignalBuilder) -> glib::subclass::Signal>(f: F) -> glib::subclass::Signal;
}

impl<T: Signal> BuildableSignal for T
where
	<Self::Return as ToValueOption>::Type: StaticType,
{
	fn builder<F: FnOnce(SignalBuilder) -> glib::subclass::Signal>(f: F) -> glib::subclass::Signal {
		let builder = glib::subclass::Signal::builder(Self::NAME)
			.param_types(<Self::Arguments as FromValues>::static_types())
			.return_type::<<Self::Return as ToValueOption>::Type>()
			.flags(Self::FLAGS);
		f(builder)
	}
}

pub trait BuildSignal: BuildableSignal {
	fn build() -> glib::subclass::Signal {
		Self::builder(|b| b.build())
	}
}

#[cfg_attr(feature = "dox", doc(notable_trait))]
pub trait Notifies<T: Signal>: ObjectType {}

#[derive(Copy, Clone, Debug)]
pub struct ConnectDetails<S = ()> {
	signal: SignalId,
	detail: Option<Quark>,
	pub run_after: bool,
	_signal: PhantomData<S>,
}

impl<S> ConnectDetails<S> {
	pub unsafe fn with_parts(signal: SignalId, detail: Option<Quark>, run_after: bool) -> Self {
		Self {
			signal,
			detail,
			run_after,
			_signal: PhantomData,
		}
	}

	pub fn normalize(&self) -> ConnectDetails<()> {
		unsafe { ConnectDetails::with_parts(self.signal(), self.detail(), self.run_after) }
	}

	pub fn signal(&self) -> SignalId {
		self.signal
	}

	pub fn detail(&self) -> Option<Quark> {
		self.detail
	}
}

impl<S: DetailedSignal> ConnectDetails<S> {
	pub fn new() -> Self {
		Self::with_after(false)
	}

	pub fn with_after(run_after: bool) -> Self {
		Self {
			signal: <S::Signal as Signal>::signal(),
			detail: S::detail(),
			run_after,
			_signal: PhantomData,
		}
	}

	pub fn set_detail(&mut self, detail: Quark) {
		assert!(self.detail.is_none());
		self.detail = Some(detail);
	}
}

impl<S: Signal> ConnectDetails<S> {
	pub fn with_detail<D: Into<Option<Quark>>>(detail: D) -> Self {
		Self {
			signal: S::signal(),
			detail: detail.into(),
			run_after: false,
			_signal: PhantomData,
		}
	}
}

impl ConnectDetails<()> {
	pub fn with_name<O: StaticType>(signal: &str, after: bool) -> Option<Self> {
		let (signal, detail) = SignalId::parse_name(signal, O::static_type(), false)?;
		Some(unsafe { Self::with_parts(signal, detail, after) })
	}
}

impl<S: DetailedSignal> From<ConnectDetails<S>> for ConnectDetails<()> {
	fn from(v: ConnectDetails<S>) -> Self {
		v.normalize()
	}
}

impl<S: DetailedSignal> From<S> for ConnectDetails<S> {
	fn from(_: S) -> Self {
		Self::new()
	}
}

pub trait ObjectSignalExt: ObjectType {
	unsafe fn handle_closure(&self, signal: &ConnectDetails, callback: &Closure) -> Result<SignalHandlerId, BoolError>;
	fn remove_handle(&self, handle: SignalHandlerId);

	fn handle<S, S_, C>(&self, signal: S_, callback: C) -> SignalHandlerId
	where
		C: Fn(&Self, S::Arguments) -> <S::Return as ToValueOption>::Type,
		S: DetailedSignal,
		S_: Into<ConnectDetails<S>>,
		Self: Notifies<S::Signal>;

	#[cfg(feature = "futures")]
	fn signal_stream<S, S_>(&self, signal: S_) -> SignalStream<Self, S::Arguments>
	where
		S: DetailedSignal,
		S_: Into<ConnectDetails<S>>,
		Self: Notifies<S::Signal>,
		<S::Return as ToValueOption>::Type: Default;
}

impl<O: ObjectType> ObjectSignalExt for O
where
	for<'a> BorrowedObject<'a, O>: FromValue<'a>,
{
	unsafe fn handle_closure(&self, signal: &ConnectDetails, callback: &Closure) -> Result<SignalHandlerId, BoolError> {
		let handle = glib::gobject_ffi::g_signal_connect_closure_by_id(
			self.as_object_ref().to_glib_none().0,
			signal.signal().into_glib(),
			signal.detail().map(|q| q.into_glib()).unwrap_or(0),
			callback.to_glib_none().0,
			signal.run_after.into_glib(),
		);
		match handle {
			0 => Err(glib::bool_error!(
				"failed to connect signal {:?} of type {:?}",
				signal,
				Self::static_type()
			)),
			handle => Ok(from_glib(handle)),
		}
	}

	fn handle<S, S_, C>(&self, signal: S_, callback: C) -> SignalHandlerId
	where
		C: Fn(&Self, S::Arguments) -> <S::Return as ToValueOption>::Type,
		S: DetailedSignal,
		S_: Into<ConnectDetails<S>>,
		Self: Notifies<S::Signal>,
	{
		let signal = signal.into();
		unsafe {
			let callback = Closure::new_unsafe(move |values| {
				let (this, args) = values.split_first().unwrap();
				let this: BorrowedObject<Self> = this.get().unwrap();
				let args = S::Arguments::from_values(args).unwrap();
				callback(&this, args).into().to_value_option()
			});
			self.handle_closure(&signal.normalize(), &callback).unwrap()
		}
	}

	fn remove_handle(&self, handle: SignalHandlerId) {
		self.disconnect(handle)
	}

	#[cfg(feature = "futures")]
	fn signal_stream<S, S_>(&self, signal: S_) -> SignalStream<Self, S::Arguments>
	where
		S: DetailedSignal,
		S_: Into<ConnectDetails<S>>,
		Self: Notifies<S::Signal>,
		<S::Return as ToValueOption>::Type: Default,
	{
		let signal = signal.into();
		SignalStream::connect(self, signal, |_, _| Default::default())
	}
}
