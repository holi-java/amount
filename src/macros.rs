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

macro_rules! impl_all_traits {
    (& $g:ident) => {
        impl_exchanger_ext!(@ $g : &$g);
    };
    (&mut $g:ident) => {
        impl_exchanger_ext!(@ $g : &mut $g);
    };
    ($ty:ident <$generic:ident>) => {
        impl_exchanger_ext!(@ $generic : $ty<$generic>);
    };
    (@ $g:ident : $ty:ty) => {
        impl<$g : Exchanger + ?Sized> Exchanger for $ty {
            type Rate = $g::Rate;
            type Err = $g::Err;

            fn rate(&self, unit: &Unit) -> Result<Self::Rate, Self::Err> {
                (**self).rate(unit)
            }

            fn sorted_units(&self) -> &[Unit] {
                (**self).sorted_units()
            }
        }

        impl<$g: ExchangerExt + ?Sized> ExchangerExt for $ty {
            fn base_unit(&self) -> Unit {
                (**self).base_unit()
            }
        }

        impl<$g: Reduce<E> + ?Sized, E: ExchangerExt> Reduce<E> for $ty {
            type Output = $g::Output;

            fn reduce(&self, exchanger: E) -> Result<Self::Output, E::Err> {
                (**self).reduce(exchanger)
            }
        }
    };
    ($($ty: ty => $g:ident),*) => {
        $(impl_all_traits!(@ $g : $ty);)*
    };
}

