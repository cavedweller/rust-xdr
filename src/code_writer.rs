use std::io::Write;

pub struct CodeWriter<'a> {
    writer: &'a mut (Write + 'a),
    indent: String,
}

impl<'a> CodeWriter<'a> {
    pub fn new(writer: &'a mut Write) -> CodeWriter<'a> {
        CodeWriter {
            writer: writer,
            indent: "".to_string(), // Two space master race
        }
    }

    pub fn indented<F>(&mut self, cb: F) where F : Fn(&mut CodeWriter) {
        cb(&mut CodeWriter {
            writer: self.writer,
            indent: format!("{}  ", self.indent),
        });
    }

    pub fn comment(&mut self, comment: &str) {
        if comment.is_empty() {
            self.write_line("//");
        } else {
            self.write_line(&format!("// {}", comment));
        }
    }

    pub fn write<S : AsRef<str>>(&mut self, line: S) {
        let s: String = [self.indent.as_ref(), line.as_ref()].concat();
        let _ = self.writer.write_all(s.as_bytes());
    }

    pub fn raw_write<S : AsRef<str>>(&mut self, text: S) {
        let _ = self.writer.write_all(text.as_ref().to_string().as_bytes());
    }

    pub fn write_line<S : AsRef<str>>(&mut self, line: S) {
        (if line.as_ref().is_empty() {
            self.writer.write_all("\n".as_bytes())
        } else {
            let s: String = [self.indent.as_ref(), line.as_ref(), "\n"].concat();
            self.writer.write_all(s.as_bytes())
        }).unwrap();
    }

    pub fn write_header(&mut self) {
        self.comment("autogenerated by xdrust");
        self.write_line("#[allow(dead_code)]");
        self.write_line("");
    }

    pub fn alias<S : AsRef<str>, F>(&mut self, name: S, cb: F)
        where F : Fn(&mut CodeWriter) {
            self.write(&format!("pub type {} = ", name.as_ref()));
            cb(self);
            self.write_line(";")
    }

    pub fn pub_enum<S : AsRef<str>, F>(&mut self, name: S, cb: F)
        where F : Fn(&mut CodeWriter) {
            self.write_line("");
            self.write_line("#[derive(Serialize, Deserialize, PartialEq, Debug)]");
            self.expr_block(&format!("pub enum {}", name.as_ref()), cb);
        }

    pub fn pub_struct<S : AsRef<str>, F>(&mut self, name: S, cb: F)
        where F : Fn(&mut CodeWriter) {
            self.write_line("");
            self.write_line("#[derive(Serialize, Deserialize, PartialEq, Debug)]");
            self.expr_block(&format!("pub struct {}", name.as_ref()), cb);
    }


    pub fn program_version_request<S: AsRef<str>, F>(&mut self, prog_name: S,
                                                     ver_num: i64, cb: F)
            where F: Fn(&mut CodeWriter) {
        self.expr_block(&format!("pub enum {}RequestV{}",
                                 prog_name.as_ref(), ver_num), cb);

    }

    pub fn version_proc_request<S1: AsRef<str>, S2: AsRef<str>>(&mut self,
                                                            name: S1,
                                                            args: &Vec<S2>) {
        self.write(name);
        if args.len() > 0 {
            self.raw_write("(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    self.raw_write(", ");
                }
                self.raw_write(&format!("{}", arg.as_ref()));
            }
            self.raw_write(")");
        }
        self.write_line(",");
    }

    pub fn program_version_response<S: AsRef<str>, F>(&mut self, prog_name: S,
                                                     ver_num: i64, cb: F)
            where F: Fn(&mut CodeWriter) {
        self.expr_block(&format!("pub enum {}ResponseV{}",
                                 prog_name.as_ref(), ver_num), cb);
    }

    pub fn version_proc_response<S1: AsRef<str>, S2: AsRef<str>>(&mut self,
                                                            name: S1,
                                                            ret: Option<S2>) {
        self.write(name);
        if let Some(s) = ret {
            self.raw_write(&format!("({})", s.as_ref()));
        }
        self.write_line(",");
    }

    pub fn namespace<S: AsRef<str>, F>(&mut self, name: S, cb: F)
            where F: Fn(&mut CodeWriter) {
        self.expr_block(&format!("pub mod {}", name.as_ref()), cb)
    }

    pub fn var_vec(&mut self, type_: &str) {
        self.write(&format!("Vec<{}>", type_));
    }

    pub fn enum_struct_decl<F>(&mut self, name: &str, cb: F)
        where F : Fn(&mut CodeWriter) {
            self.write_line(&format!("{} {{", name));
            self.indented(cb);
            self.write_line("},");
    }

    pub fn enum_decl(&mut self, name: &str, val: &str) {
        // TODO is Option<&str> cleaner?
        if val == "" {
            self.write_line(&format!("{},", name));
        } else {
            self.write_line(&format!("{} = {},", name, val));
        }
    }

    pub fn pub_field_decl(&mut self, name: &str, field_type: &str) {
        self.write_line(&format!("pub {}: {},", name, field_type));
    }

    pub fn field_decl(&mut self, name: &str, field_type: &str) {
        self.write_line(&format!("{}: {},", name, field_type));
    }

    pub fn expr_block<F>(&mut self, prefix: &str, cb: F)
        where F : Fn(&mut CodeWriter) {
            self.block(&format!("{} {{", prefix), "}", cb);
    }

    pub fn block<F>(&mut self, first_line: &str, last_line: &str, cb: F)
        where F : Fn(&mut CodeWriter) {
            self.write_line(first_line);
            self.indented(cb);
            self.write_line(last_line);
    }
}
