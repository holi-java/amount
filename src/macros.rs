macro_rules! impl_binop {
    (@$trait:ident::$method:ident -> $output:ty; $target:ty > $from:ty ; $($tg:ident),*; $($ug:ident),*) => {
        impl <$($tg,)* $($ug,)*> $trait<$from> for $target {
            type Output = $output;
            fn $method(self, addend: $from) -> Self::Output {
                Sum(self, addend)
            }
        }

        impl <$($tg,)* $($ug: Clone,)*> $trait<&$from> for $target {
            type Output = $output;
            fn $method(self, addend: &$from) -> Self::Output {
                Sum(self, addend.clone())
            }
        }

        impl <$($tg: Clone,)* $($ug,)*> $trait<$from> for &$target {
            type Output = $output;
            fn $method(self, addend: $from) -> Self::Output {
                Sum(self.clone(), addend)
            }
        }

        impl <$($tg: Clone,)* $($ug: Clone,)*> $trait<&$from> for &$target {
            type Output = $output;

            fn $method(self, addend: &$from) -> Self::Output {
                Sum(self.clone(), addend.clone())
            }
        }
    };
}

macro_rules! impl_addop {
    ($($target:ident$(<$($tg:ident),*>)? => $from:ident$(<$($ug:ident),*>)?),*) => {
        $(
            impl_binop!(
                @Add::add -> Sum<$target$(<$($tg),*>)?, $from$(<$($ug),*>)?>;
                $target$(<$($tg),*>)? > $from$(<$($ug),*>)?; $($($tg),*)?; $($($ug),*)?
            );
        )*
    };
}

macro_rules! impl_mulop {
    (<$($T:ident$(:$t:path)?),+> $trait:ty => $ty:ty: [$($constraints:tt)*] { ($self:ident, $arg: ident) -> $block:block }) => {
        impl <$($T $(: $t)?),+> $trait for $ty where $($constraints)* {
            type Output = $ty;
            fn mul(#[allow(unused_mut)] mut $self, $arg: T) -> Self::Output  $block
        }

        impl <$($T $(: $t)?),+> $trait for &$ty where $($constraints)* {
            type Output = $ty;
            fn mul(self, multiplier: T) -> Self::Output {
                self.clone() * multiplier
            }
        }

    };
}

macro_rules! parse_generic_types {
    ($macro:ident!()) => {};
    ($macro:ident!( &$T:ident $(,$($tt: tt)*)? )) => {
        $macro!( (<$T>, &$T) );
        parse_generic_types!($macro!( $($($tt)*)? ));
    };
    ($macro:ident!( &mut $T:ident $(,$($tt: tt)*)? )) => {
        $macro!( (<$T>, &mut $T) );
        parse_generic_types!($macro!( $($($tt)*)? ));
    };
    ($macro:ident!( $ty:ident<$T:ident> $(,$($tt: tt)*)? )) => {
        $macro!( (<$T>, $ty<$T>) );
        parse_generic_types!($macro!( $($($tt)*)? ));
    };
}

macro_rules! impl_all_traits {
    ((<$T:ident>, $ty:ty)) => {
        impl<$T: Exchanger + ?Sized> Exchanger for $ty {
            type Rate = $T::Rate;
            type Err = $T::Err;

            fn rate(&self, unit: &Unit) -> Result<Self::Rate, Self::Err> {
                (**self).rate(unit)
            }

            fn sorted_units(&self) -> &[$crate::traits::UnitRate<Self::Rate>] {
                (**self).sorted_units()
            }

            fn base_unit(&self) -> &Unit {
                (**self).base_unit()
            }
        }

        impl<$T: Reduce<E> + ?Sized, E: Exchanger> Reduce<E> for $ty {
            type Output = $T::Output;

            fn reduce(&self, exchanger: E) -> Result<Self::Output, E::Err> {
                (**self).reduce(exchanger)
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! count {
    ($tt: tt) => (1);
    ($($tt:tt)*) => {
        0 $(+ $crate::count!($tt))*
    };
}

/// ```rust
/// # use crate::amount::define_exchanger;
/// define_exchanger!{
///     #[base_unit="g"]
///     #[derive(Debug, Clone)]
///     pub Weight {
///         t = 1_000_000,
///         kg = 1_000,
///         jin = 500
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_exchanger {
    (#[base_unit=$base:literal] $(#[$meta:meta])* $vis:vis $ty: ident { $($fields: ident = $rates: literal),* }) => {
        $(#[$meta])*
        $vis struct $ty {
            table: $crate::Table
        }

        impl $ty {
            const UNIT_RATES: [(&'static str, u64); $crate::count!($($rates)*) + 1] =
                [$((stringify!($fields), $rates),)* ($base, 1)];

            #[inline]
            #[cold]
            #[allow(dead_code)]
            pub fn units() -> impl Iterator<Item = &'static str> {
                Self::unit_rates().map(|(unit, _)| unit).cloned()
            }

            pub fn unit_rates() -> impl Iterator<Item = &'static (&'static str, u64)> {
                Self::UNIT_RATES.iter().filter(|(unit, _)| *unit != $base).chain(::core::iter::once(&($base, 1)))
            }

            pub fn base<T: Into<$crate::Unit>>(unit: T) -> Result<Self, $crate::Error> {
                let base = unit.into();
                let mut units = Vec::with_capacity(Self::UNIT_RATES.len());
                for &(test, rate) in Self::unit_rates() {
                    units.push(($crate::Unit::new(test), rate));
                    if test == base.key {
                        units.iter_mut().for_each(|(_, origin)| *origin /= rate);
                        units.shrink_to_fit();
                        return Ok(Self { table: $crate::Table::new(base, units) });
                    }
                }
                Err($crate::Error::NotFound(base))
            }
        }


        impl Default for $ty {
            #[inline]
            #[cold]
            fn default() -> Self {
                Self::base($base).unwrap()
            }
        }


        impl $crate::Exchanger for $ty {
            type Rate = u64;

            type Err = $crate::Error;

            fn rate(&self, unit: &$crate::Unit) -> Result<Self::Rate, Self::Err> {
                self.table.rate(unit)
            }

            #[inline]
            #[cold]
            fn sorted_units(&self) -> &[($crate::Unit, Self::Rate)] {
                self.table.sorted_units()
            }

            #[inline]
            #[cold]
            fn base_unit(&self) -> &$crate::Unit {
                self.table.base_unit()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{
        test::{g, kg},
        Amount, Error, Exchanger, ExchangerExt, Unit,
    };
    use std::assert_matches::assert_matches;

    define_exchanger!(
        #[base_unit="MG"]
        #[derive(Debug)]
        Weight {
            T = 1_000_000_000,
            KG = 1_000_000,
            JIN = 500_000,
            G = 1_000
        }
    );

    #[test]
    fn define_exchanger_using_macros() {
        assert_eq!(
            Weight::units().collect::<Vec<_>>(),
            ["T", "KG", "JIN", "G", "MG"]
        );

        let weight = Weight::default();
        {
            let exchanger = &weight as &dyn Exchanger<Rate = u64, Err = Error>;
            assert_eq!(
                exchanger.sorted_units(),
                [
                    (Unit::new("T"), 1_000_000_000),
                    (Unit::new("KG"), 1_000_000),
                    (Unit::new("JIN"), 500_000),
                    (Unit::new("G"), 1_000),
                    (Unit::new("MG"), 1),
                ]
            );

            assert_eq!(exchanger.rate(&Unit::new("T")).unwrap(), 1_000_000_000);
            assert_eq!(exchanger.rate(&Unit::new("MG")).unwrap(), 1);
            assert_matches!(exchanger.rate(&Unit::new("BAG")), Err(Error::NotFound(unit)) if unit.key == "BAG");
        }
        {
            let exchanger = &weight as &dyn ExchangerExt<Rate = u64, Err = Error>;
            assert_eq!(*exchanger.base_unit(), Unit::new("MG"));
        }
    }

    #[test]
    fn weight_with_specified_base_unit() {
        let weight = Weight::base("JIN").unwrap();
        assert_eq!(
            weight.sorted_units(),
            [
                (Unit::new("T"), 2000),
                (Unit::new("KG"), 2),
                (Unit::new("JIN"), 1)
            ]
        );
        assert_eq!(*weight.base_unit(), Unit::new("JIN"));

        assert_matches!(
            weight.reduce(Amount::new(500, Unit::new("G"))),
            Err(Error::NotFound(_))
        );
        assert_eq!(
            weight.reduce(Amount::new(5, Unit::new("KG"))).unwrap(),
            Amount::new(5 * 2, Unit::new("JIN"))
        );
    }

    #[test]
    fn weight_with_invalid_base_unit() {
        let result = Weight::base("bag");
        assert_matches!(result, Err(Error::NotFound(unit)) if unit.key == "bag");
    }

    #[test]
    fn define_exchanger_with_base_unit_rate() {
        define_exchanger! {
            #[base_unit = "g"]
            Weight {
                kg = 1_000,
                g = 1
            }
        }
        let weight = Weight::default();

        assert_eq!(weight.sorted_units().to_vec(), [(kg(), 1_000), (g(), 1)]);
        assert_eq!(Weight::units().collect::<Vec<_>>(), ["kg", "g"]);
    }

    mod visibility {
        mod private {
            define_exchanger!(#[base_unit = "g"] pub Weight {
                g = 1
            });
        }

        #[test]
        fn accessible() {
            #[allow(unused)]
            use private::Weight;
        }
    }
}
