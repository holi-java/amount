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

            fn units(&self) -> &[$crate::traits::UnitRate<Self::Rate>] {
                (**self).units()
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

#[macro_export]
macro_rules! define_exchanger {
    (#[base_unit=$base:literal] $(#[$meta:meta])* $vis:vis $ty: ident { $($fields: ident = $rates: literal),* }) => {
        $(#[$meta])*
        $vis struct $ty {
            units: Vec<($crate::Unit, u64)>
        }

        impl $ty {
            const N: usize = $crate::count!($($rates)*) + 1;
            const RATES: [u64; Self::N] = [$($rates,)* 1];
            const UNITS: [&'static str; Self::N] = [$(stringify!($fields),)* $base];

            #[inline]
            #[cold]
            #[allow(dead_code)]
            pub fn units() -> &'static [&'static str] {
                &Self::UNITS
            }

            pub fn base<T: Into<$crate::Unit>>(unit: T) -> Result<Self, $crate::Error> {
                let base = unit.into();
                let mut units = Vec::with_capacity(Self::N);
                for (&test, rate) in Self::UNITS.iter().zip(&Self::RATES) {
                    units.push(($crate::Unit::new(test), *rate));
                    if test == base.key {
                        units.iter_mut().for_each(|(_, origin)| *origin /= rate);
                        units.shrink_to_fit();
                        return Ok(Self { units });
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
                for (test, rate) in self.units.iter() {
                    if test == unit {
                        return Ok(*rate);
                    }
                }
                Err($crate::Error::NotFound(unit.clone()))
            }

            #[inline]
            #[cold]
            fn units(&self) -> &[($crate::Unit, Self::Rate)] {
                &self.units
            }

            #[inline]
            #[cold]
            fn base_unit(&self) -> &$crate::Unit {
                if let Some((unit, _)) = self.units.iter().find(|(_, rate)| *rate == 1) {
                    return unit;
                }
                else {
                    todo!()
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{Amount, Error, Exchanger, ExchangerExt, Unit};
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
        assert_eq!(Weight::units(), ["T", "KG", "JIN", "G", "MG"]);

        let weight = Weight::default();
        {
            let exchanger = &weight as &dyn Exchanger<Rate = u64, Err = Error>;
            assert_eq!(
                exchanger.units(),
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
            weight.units(),
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
