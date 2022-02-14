mod config;
use object::read::File;


pub struct TestBinary<'data> {
    pub file: File<'data>,
}

impl<'data> TestBinary<'data> {
    pub fn new(file: File<'data>) -> Self {
        Self {
            file
        }
    }



}

