/// A helper that impls [Signal](crate::Signal) and its related traits.
///
/// ## Syntax
///
/// ```
/// use glib_signal::{impl_signal, SignalFlags};
/// # use glib::Object as AnObject;
/// # #[derive(Copy, Clone, Debug)] struct SignalType;
/// impl_signal! { impl Notifies<"signal-name" as SignalType> for AnObject {
///     //impl {const SIGNAL_NAME}; // provide a convenient accessor for the default signal handler
///     impl BuildSignal; // provide a default impl to facilitate GObject type construction
///     FLAGS = SignalFlags::NO_RECURSE; // optionally specify flags for the signal when building
///     fn(&self, String) // finally, specify the callback handler signature (with optional return type)
/// } }
/// ```
///
/// ## Implements
///
/// - [Notifies<`SignalType`>](crate::Notifies) for `AnObject`
/// - [Signal](crate::Signal) for `SignalType`
/// - [BuildSignal](crate::BuildSignal) for `SignalType`, for use with
///   [glib::ObjectImpl](glib::subclass::object::ObjectImpl::signals) (opt-in)
#[macro_export]
macro_rules! impl_signal {
	(impl Notifies<$signal_str:literal as $signal:path> for $obj:path {
		$(impl $imp:tt;)*
		$(FLAGS = $flags:expr;)*
		fn $($handler:tt)*
	}) => {
		impl $crate::Signal for $signal {
			type Object = $obj;
			$crate::_impl_signal_private! { @line Handler fn $($handler)* }
			$(
				$crate::_impl_signal_private! { @line FLAGS $flags }
			)*

			const NAME: &'static str = $signal_str;
		}

		impl $crate::Notifies<$signal> for $obj { }
		$(
			$crate::_impl_signal_private! { @impl ($signal) ($obj) $imp }
		)*
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! _impl_signal_private {
	(@ty $ty:ty) => { $ty };
	(@args (&self $(,$args:ty)*)) => {
		($($args,)*)
	};
	(@line Handler fn $args:tt -> $($res:tt)+) => {
		type Arguments = $crate::_impl_signal_private! { @args $args };
		type Return = $crate::_impl_signal_private! { @ty $($res)* };
	};
	(@line Handler fn $args:tt) => {
		type Arguments = $crate::_impl_signal_private! { @args $args };
		type Return = $crate::PrimitiveValue<()>;
	};
	(@line FLAGS $flags:expr) => {
		const FLAGS: $crate::glib::SignalFlags = $flags;
	};
	(@impl ($signal:path) ($obj:path) BuildSignal) => {
		impl $crate::BuildSignal for $signal { }
	};
	(@impl ($signal:path) ($obj:path) {const $signal_const:ident}) => {
		impl $obj {
			pub const $signal_const: $signal = $signal;
		}
	};
}

/// The same syntax as [impl_signal], but also defines the corresponding signal struct.
#[macro_export]
macro_rules! def_signal {
	(
		$(#[$meta:meta])*
		impl Notifies<$signal_str:literal as $signal:tt> for $obj:path { $($inner:tt)* }
	) => {
		$(
			#[$meta]
		)*
		#[derive(Copy, Clone, Debug)]
		pub struct $signal;

		$crate::impl_signal! {
			impl Notifies<$signal_str as $signal> for $obj {
				$($inner)*
			}
		}
	};
}
