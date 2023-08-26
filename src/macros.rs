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

macro_rules! impl_exchanger {
    (&$ty:ty) => {impl_exchanger!(@&);};
    (&mut $ty:ty) => {impl_exchanger!(@&mut );};
    (@&$($mut:ident)?) => {
        impl<E: Exchanger> Exchanger for &$($mut)? E {
            type Rate = E::Rate;
            type Err = E::Err;

            fn rate(&self, unit: &Unit) -> Result<Self::Rate, Self::Err> {
                (**self).rate(unit)
            }
        }

        impl<E: ExchangerExt> ExchangerExt for &$($mut)? E {
            fn base_unit(&self) -> Unit {
                (**self).base_unit()
            }

            fn sorted_units(&self) -> &[Unit] {
                (**self).sorted_units()
            }
        }
    };
}

macro_rules! impl_reduce {
    (&$ty:ty) => {impl_reduce!(@&);};
    (&mut $ty:ty) => {impl_reduce!(@&mut );};
    (@&$($mut:ident)?) => {
        impl<T: Reduce<E>, E: ExchangerExt> Reduce<E> for &$($mut)? T {
            type Output = T::Output;

            fn reduce(&self, exchanger: E) -> Result<Self::Output, E::Err> {
                (**self).reduce(exchanger)
            }
        }
    };
}

macro_rules! impl_mulop {
    (impl <$($g:ident$(:$t:path)?),+> $trait:ty => $target:ty |$self:ident, $arg: ident|$block:block $($tt:tt)*) => {
        impl <$($g $(: $t)?),+> $trait for $target $($tt)* {
            type Output = $target;
            fn mul(#[allow(unused_mut)] mut $self, $arg: T) -> Self::Output  $block
        }


        impl <$($g $(: $t)?),+> $trait for &$target $($tt)* {
            type Output = $target;
            fn mul(self, multiplier: T) -> Self::Output {
                self.clone() * multiplier
            }
        }

    };
}
