use crate::asm::wrapper_types::Imm12;
use crate::asm::Mask;
#[macro_export]
macro_rules! instruction {
    (size $size:ty;
     $(
         $id:ident : {
            $(
                $field_id:ident $(as $representation:ty)? : $type:ty : $start:literal -> $end:literal $($expr:ident)?
            ),*
        }
    ),*
    ) => {
        $(
            paste!{
                #[doc = "Instruction " [<$id>] "\n\n"]
                #[doc = "Contains the following fields:\n"]
                $(
                    #[doc = "- " [<$field_id>] " of type " [<$type>] " from bit " [<$start>] " to bit " [<$end>] "\n"]
                )+
                #[derive(Debug)]
                pub struct $id {
                    $(
                        #[doc = "bit " [<$start>] " to " [<$end>]]
                        pub $field_id:$type,
                    )+
                }
            }


            impl Parse for $id{
                type Target = Self;
                fn parse<T: crate::Stream>(iter: &mut T) -> Result<Self::Target, crate::ParseError>
                where
                    Self: Sized {
                    let word: $size = match iter.consume::<1>(){
                        Some(buff) => Ok(buff[0]),
                        None => Err(ParseError::Invalid16Bit(stringify!($id))),
                    }?;
                    $(
                        let $field_id:$type = instruction!($size;word $(as $representation)?; $start -> $end $($expr)?);

                    )+
                    Ok(Self{
                        $(
                            $field_id: $field_id,
                        )+
                    })
                }
            }
        )+
    };

    (
        $size:ty; $word:ident $(as $representation:ty)?; $start:literal -> $end:literal $($expr:ident)?
    ) => {
            {
                (($word as $size).mask::<$start,$end>() $(as $representation)?)$(.$expr()?)?

            }
    };

    (
    size $size:ty; $table:ident contains
        $(
            $($id:ident : {
                $(
                    $field_id:ident $(as $representation:ty)?: $type:ty : $start:literal -> $end:literal $($expr:ident)?

                ),*
            })?
            $(
                -> $table_id:ident
            )?
        ),*
    ) => {
        paste!{
            #[derive(Debug)]
            pub enum $table{
                $(
                    $($id($id),)?
                    $(
                        #[doc = "Externally defined instruction or set of instructions [`"  [<$table_id>]  "`]"]
                        [<Subtable $table_id>]($table_id),
                    )?
                )+
            }
        }
        $(

            $(
                paste!{
                    #[doc = "Instruction " [<$id>] " from table " [<$table>] "\n\n"]
                    #[doc = "Contains the following fields:\n"]
                    $(
                        #[doc = "- " [<$field_id>] " of type " [<$type>] " from bit " [<$start>] " to bit " [<$end>] "\n"]
                    )*
                    #[derive(Debug)]
                    pub struct $id {
                        $(
                            #[doc = "bit " [<$start>] " to " [<$end>]]
                            pub $field_id:$type,
                        )*
                    }
                }


                impl Parse for $id{
                    type Target = Self;
                    fn parse<T: crate::Stream>(iter: &mut T) -> Result<Self::Target, crate::ParseError>
                    where
                        Self: Sized {
                        // Consume a word from the buffer
                        let word:$size = match iter.consume::<1>(){
                            Some(buff) => Ok(buff[0]),
                            None => Err(ParseError::Invalid16Bit(stringify!($id))),
                        }?;
                        println!("Checking word {word:#018b}");
                        $(
                            let $field_id:$type = instruction!($size; word $(as $representation)?; $start -> $end $($expr)?);
                        )*
                        let ret = Self{
                            $(
                                $field_id: $field_id,
                            )*
                        };
                        println!("Parsed {:?}",ret);
                        Ok(ret)
                    }
                }
            )?
        )*
    }
}
#[macro_export]
macro_rules! combine {
    ($first_id:ident:$($id:ident,$size:literal):*,$ret_ty:ty) => {
        {

            let mut counter:usize = {
                $($size+)*0
            };
            let mut sum: $ret_ty = $first_id as $ret_ty << counter;
            #[allow(unused_assignments)]
            {
                $(
                    counter = counter - $size;
                    sum |= (($id as $ret_ty) << counter) as $ret_ty;
                )*
            }
            sum
        }
    };
}

#[cfg(test)]
mod test {

    #[test]
    fn test() {
        let i: u8 = 1;
        let imm2: u8 = 2;
        let imm3: u8 = 4;
        let res: u32 = combine!(i:imm2,2:imm3,3,u32);
        println!("{res:#010b}");
        assert_eq!(0b110100, res);
        let zero = 0;
        let res: u32 = combine!(i:zero,2,u32);
        assert_eq!(0b100, res)
    }
}