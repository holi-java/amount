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
    (impl <$($T:ident$(:$t:path)?),+> $trait:ty => $target:ty |$self:ident, $arg: ident|$block:block $($tt:tt)*) => {
        impl <$($T $(: $t)?),+> $trait for $target $($tt)* {
            type Output = $target;
            fn mul(#[allow(unused_mut)] mut $self, $arg: T) -> Self::Output  $block
        }


        impl <$($T $(: $t)?),+> $trait for &$target $($tt)* {
            type Output = $target;
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
        }

        impl<$T: ExchangerExt + ?Sized> ExchangerExt for $ty {
            fn base_unit(&self) -> Unit {
                (**self).base_unit()
            }
        }

        impl<$T: Reduce<E> + ?Sized, E: ExchangerExt> Reduce<E> for $ty {
            type Output = $T::Output;

            fn reduce(&self, exchanger: E) -> Result<Self::Output, E::Err> {
                (**self).reduce(exchanger)
            }
        }
    };
}
