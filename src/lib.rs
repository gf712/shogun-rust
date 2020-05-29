mod bindings;

pub mod shogun {

    mod details {
        use crate::bindings;
        use std::ffi::CStr;
        pub fn sgobject_to_string<T>(obj: *const T) -> &'static str {
            let c_repr =
                unsafe { bindings::to_string(obj as *const _ as *const bindings::sgobject_t) };
            let repr = unsafe { CStr::from_ptr(c_repr) };
            repr.to_str()
                .expect("Failed to get SGObject representation")
        }

        pub fn handle_result(result: &bindings::Result) -> Result<(), String> {
            unsafe {
                match result {
                    bindings::Result {
                        return_code: bindings::RETURN_CODE_ERROR,
                        error: msg,
                    } => {
                        let c_error_str = CStr::from_ptr(*msg);
                        Err(format!("{}", c_error_str.to_str().expect("Failed to get error")))
                    }
                    bindings::Result {
                        return_code: bindings::RETURN_CODE_SUCCESS,
                        error: _,
                    } => Ok(()),
                    _ => Err("Unexpected return.".to_string())
                }
            }
        }
    }

    use crate::bindings;
    use shogun_rust_procedural::SGObject;
    use std::ffi::{CStr, CString};
    use std::fmt;
    use std::str::Utf8Error;
    extern crate ndarray;
    use ndarray::Array2;

    trait SGObject: fmt::Display {
        fn to_string(&self) -> &str;
    }

    pub trait SGObjectPut {
        fn sgobject_put(&self, obj: *mut bindings::sgobject, name: &'static str) -> Result<(), String>;
    }

    macro_rules! add_sgobject_put_type {
        ($put_type:ty, $enum_value:expr) => {
            impl SGObjectPut for $put_type {
                fn sgobject_put(&self, obj: *mut bindings::sgobject, parameter_name: &'static str) -> Result<(), String> {
                    unsafe {
                        let c_string = CString::new(parameter_name).expect("CString::new failed");
                        let type_erased_parameter = std::mem::transmute::<&$put_type, *const std::ffi::c_void>(&self);
                        details::handle_result(&bindings::sgobject_put(obj, c_string.as_ptr(), type_erased_parameter, $enum_value))
                    }
                }
            }
        }
    }

    add_sgobject_put_type!(i32, bindings::TYPE_INT32);
    add_sgobject_put_type!(i64, bindings::TYPE_INT64);
    add_sgobject_put_type!(f32, bindings::TYPE_FLOAT32);
    add_sgobject_put_type!(f64, bindings::TYPE_FLOAT64);

    pub struct Version {
        version_ptr: *mut bindings::version_t,
    }

    #[derive(SGObject)]
    pub struct Machine {
        ptr: *mut bindings::sgobject,
    }

    #[derive(SGObject)]
    pub struct Kernel {
        ptr: *mut bindings::sgobject,
    }

    #[derive(SGObject)]
    pub struct Distance {
        ptr: *mut bindings::sgobject,
    }

    #[derive(SGObject)]
    pub struct Features {
        ptr: *mut bindings::sgobject,
    }

    #[derive(SGObject)]
    pub struct File {
        ptr: *mut bindings::sgobject,
    }

    #[derive(SGObject)]
    pub struct CombinationRule {
        ptr: *mut bindings::sgobject,
    }

    #[derive(SGObject)]
    pub struct Labels {
        ptr: *mut bindings::sgobject,
    }

    pub trait MatrixToFeatures {
        fn create_features_from_matrix(&self) -> Result<Features, String>;
    }

    macro_rules! add_matrix_type {
        ($array_type:ty, $enum_value:expr) => {
            impl MatrixToFeatures for Array2<$array_type> {
                fn create_features_from_matrix(&self) -> Result<Features, String> {
                    let n_rows = self.nrows();
                    let n_cols = self.ncols();
                    unsafe {
                        let data = self.as_ptr();
                        let type_erased_matrix = std::mem::transmute::<*const $array_type, *const std::ffi::c_void>(data);
                        match bindings::create_features_from_data(type_erased_matrix, n_rows as u32, n_cols as u32, $enum_value) {
                            bindings::sgobject_result { return_code: bindings::RETURN_CODE_SUCCESS,
                                result: bindings::sgobject_result_ResultUnion { result: ptr } } => {
                                  Ok(Features { ptr })
                              },
                            bindings::sgobject_result { return_code: bindings::RETURN_CODE_ERROR,
                                result: bindings::sgobject_result_ResultUnion { error: msg } } => {
                                let c_error_str = CStr::from_ptr(msg);
                                Err(format!("{}", c_error_str.to_str().expect("Failed to get error")))
                            },
                            _ => Err("Unexpected return.".to_string())
                        }
                    }
                }
            }
            impl SGObjectPut for Array2<$array_type> {
                fn sgobject_put(&self, obj: *mut bindings::sgobject, parameter_name: &'static str) -> Result<(), String> {
                    let n_rows = self.nrows() as u32;
                    let n_cols = self.ncols() as u32;
                    unsafe {
                        let data = self.as_ptr();
                        let c_string = CString::new(parameter_name).expect("CString::new failed");
                        let type_erased_matrix = std::mem::transmute::<*const $array_type, *const std::ffi::c_void>(data);
                        details::handle_result(&bindings::sgobject_put_array(obj, c_string.as_ptr(), type_erased_matrix, n_rows, n_cols, $enum_value))
                    }
                }
            }       
        };
    }

    add_matrix_type!(f32, bindings::TYPE_FLOAT32);
    add_matrix_type!(f64, bindings::TYPE_FLOAT64);
    add_matrix_type!(i32, bindings::TYPE_INT32);
    add_matrix_type!(i64, bindings::TYPE_INT64);

    impl Features {
        pub fn from_array<T>(array: &Array2<T>) -> Result<Features, String>
        where Array2<T>: MatrixToFeatures {
            array.create_features_from_matrix()
        }

        pub fn from_file(file: &File) -> Result<Features, String> {
            unsafe {
                let c_ptr = bindings::create_features_from_file(file.ptr);
                match c_ptr {
                    bindings::sgobject_result { return_code: bindings::RETURN_CODE_SUCCESS,
                                      result: bindings::sgobject_result_ResultUnion { result: ptr } } => {
                                        Ok(Features { ptr })
                                    },
                    bindings::sgobject_result { return_code: bindings::RETURN_CODE_ERROR,
                        result: bindings::sgobject_result_ResultUnion { error: msg } } => {
                        let c_error_str = CStr::from_ptr(msg);
                        Err(format!("{}", c_error_str.to_str().expect("Failed to get error")))
                    },
                    _ => Err(format!("Unexpected return."))
                }
            }
        }
    }

    impl Kernel {
        pub fn init(&mut self, lhs: &Features, rhs: &Features) -> Result<(), String> {
            unsafe {
                details::handle_result(&bindings::init_kernel(self.ptr, lhs.ptr, rhs.ptr))
            }
        }
    }

    impl Machine {
        pub fn train(&mut self, features: &Features) -> Result<(), String> {
            unsafe {
                details::handle_result(&bindings::train_machine(self.ptr, features.ptr))
            }
        }
        pub fn apply(&self, features: &Features) -> Result<Labels, String> {
            unsafe {
                let c_ptr = bindings::apply_machine(self.ptr, features.ptr);
                match c_ptr {
                    bindings::sgobject_result { return_code: bindings::RETURN_CODE_SUCCESS,
                                      result: bindings::sgobject_result_ResultUnion { result: ptr } } => {
                                        Ok(Labels { ptr })
                                    },
                    bindings::sgobject_result { return_code: bindings::RETURN_CODE_ERROR,
                        result: bindings::sgobject_result_ResultUnion { error: msg } } => {
                        let c_error_str = CStr::from_ptr(msg);
                        Err(format!("{}", c_error_str.to_str().expect("Failed to get error")))
                    },
                    _ => Err(format!("Unexpected return."))
                }
            }
        }

        pub fn apply_multiclass(&self, features: &Features) -> Result<Labels, String> {
            unsafe {
                let c_ptr = bindings::apply_multiclass_machine(self.ptr, features.ptr);
                match c_ptr {
                    bindings::sgobject_result { return_code: bindings::RETURN_CODE_SUCCESS,
                                      result: bindings::sgobject_result_ResultUnion { result: ptr } } => {
                                        Ok(Labels { ptr })
                                    },
                    bindings::sgobject_result { return_code: bindings::RETURN_CODE_ERROR,
                        result: bindings::sgobject_result_ResultUnion { error: msg } } => {
                        let c_error_str = CStr::from_ptr(msg);
                        Err(format!("{}", c_error_str.to_str().expect("Failed to get error")))
                    },
                    _ => Err(format!("Unexpected return."))
                }
            }
        }
    }

    impl File {
        pub fn read_csv(filepath: String) -> Result<Self, String> {
            unsafe {
                let c_string = CString::new(filepath).expect("CString::new failed");
                let c_ptr = bindings::read_csvfile(c_string.as_ptr());
                match c_ptr {
                    bindings::sgobject_result { return_code: bindings::RETURN_CODE_SUCCESS,
                                      result: bindings::sgobject_result_ResultUnion { result: ptr } } => {
                                        Ok(File { ptr })
                                    },
                    bindings::sgobject_result { return_code: bindings::RETURN_CODE_ERROR,
                        result: bindings::sgobject_result_ResultUnion { error: msg } } => {
                        let c_error_str = CStr::from_ptr(msg);
                        Err(format!("{}", c_error_str.to_str().expect("Failed to get error")))
                    },
                    _ => Err(format!("Unexpected return."))
                }
            }
        }
    }

    impl Labels {
        pub fn from_file(file: &File) -> Result<Labels, String> {
            unsafe {
                let c_ptr = bindings::create_labels_from_file(file.ptr);
                match c_ptr {
                    bindings::sgobject_result { return_code: bindings::RETURN_CODE_SUCCESS,
                                    result: bindings::sgobject_result_ResultUnion { result: ptr } } => {
                                        Ok(Labels { ptr })
                                    },
                    bindings::sgobject_result { return_code: bindings::RETURN_CODE_ERROR,
                        result: bindings::sgobject_result_ResultUnion { error: msg } } => {
                        let c_error_str = CStr::from_ptr(msg);
                        Err(format!("{}", c_error_str.to_str().expect("Failed to get error")))
                    },
                    _ => Err(format!("Unexpected return."))
                }
            }
        }
    }

    impl Version {
        pub fn new() -> Self {
            Version {
                version_ptr: unsafe { bindings::create_version() },
            }
        }

        pub fn main_version(&self) -> Result<String, String> {
            let char_ptr = unsafe { bindings::get_version_main(self.version_ptr) };
            let c_str = unsafe { CStr::from_ptr(char_ptr) };
            match c_str.to_str() {
                Err(x) => Err(x.to_string()),
                Ok(x) => Ok(x.to_string()),
            }
        }
    }


    pub fn set_num_threads(n_threads: i32) {
        unsafe {bindings::set_parallel_threads(n_threads)};
    }

    impl Drop for Version {
        fn drop(&mut self) {
            unsafe { bindings::destroy_version(self.version_ptr) };
        }
    }
}
