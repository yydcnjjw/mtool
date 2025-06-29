use std::{
    ffi,
    ffi::OsString,
    os::windows::prelude::{
        OsStrExt,
        OsStringExt,
    },
    path::Path,
    ptr,
    slice,
};

use windows::core::{
    Error,
    Result,
    PCWSTR,
    w as pcwstr,
};

use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW,
    GetFileVersionInfoW,
    VerQueryValueW,
};

/// Represents version information for a file.
///
/// This struct contains various fields that provide detailed information
/// about the file, such as its description and version number, company that
/// produced it, and other metadata.
/// 
/// This struct uses the idiomatic [`String`] for its string fields. This means
/// that any possibly ill-formed UTF-16 data that may be present in the version
/// information of the file will be replaced with the Unicode replacement
/// character (?) when converting to a [`String`]. If you need to preserve such
/// data, use [`VersionInfoOs`] instead.
/// 
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[non_exhaustive]
pub struct VersionInfo {
    /// The comments associated with the file.
    pub comments: String,
    /// The name of the company that produced the file.
    pub company_name: String,
    /// The description of the file.
    pub file_description: String,
    /// The file version number.
    pub file_version: String,
    /// The internal name of the file, if one exists.
    pub internal_name: String,
    /// The copyright notices that apply to the specified file.
    pub legal_copyright: String,
    /// The trademarks and registered trademarks that apply to the file.
    pub legal_trademarks: String,
    /// The name the file was created with.
    pub original_filename: String,
    /// The name of the product this file is distributed with.
    pub product_name: String,
    /// The version of the product this file is distributed with.
    pub product_version: String,
    /// The private build information for the file.
    pub private_build: String,
    /// The special build information for the file.
    pub special_build: String,
}

impl VersionInfo {
    /// Retrieves version information from the specified file.
    /// 
    /// As [`VersionInfo`] uses [`String`] for its string fields, any possibly
    /// ill-formed UTF-16 data that may be present in the version information
    /// of the file will be replaced with the Unicode replacement character (?)
    /// when converting to a [`String`]. If you need to preserve such data, use
    /// [`VersionInfoOs::from_file`] instead.
    /// 
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The file does not exist.
    /// - The file is not accessible.
    /// - The version information cannot be retrieved.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate_name::VersionInfo;
    ///
    /// let info = VersionInfo::from_file("path/to/your/file.exe")
    ///     .expect("Failed to retrieve version information");
    ///
    /// println!("File description: {}", info.file_description);
    /// println!("File version: {}", info.file_version);
    /// ```
    pub fn from_file<P: AsRef<Path>>(file_name: P) -> Result<Self> {
        let info = VersionInfoOs::from_file(file_name)?;
        Ok(Self {
            comments: info.comments.to_string_lossy().into_owned(),
            company_name: info.company_name.to_string_lossy().into_owned(),
            file_description: info.file_description.to_string_lossy().into_owned(),
            file_version: info.file_version.to_string_lossy().into_owned(),
            internal_name: info.internal_name.to_string_lossy().into_owned(),
            legal_copyright: info.legal_copyright.to_string_lossy().into_owned(),
            legal_trademarks: info.legal_trademarks.to_string_lossy().into_owned(),
            original_filename: info.original_filename.to_string_lossy().into_owned(),
            product_name: info.product_name.to_string_lossy().into_owned(),
            product_version: info.product_version.to_string_lossy().into_owned(),
            private_build: info.private_build.to_string_lossy().into_owned(),
            special_build: info.special_build.to_string_lossy().into_owned(),
        })
    }
}

/// Represents version information for a file.
///
/// This struct contains various fields that provide detailed information
/// about the file, such as its description and version number, company that
/// produced it, and other metadata.
/// 
/// This struct is similar to [`VersionInfo`], but it uses [`OsString`] instead
/// of [`String`] for its string fields to preserve any possibly ill-formed
/// UTF-16 data that may be present in the version information of the file.
/// 
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[non_exhaustive]
pub struct VersionInfoOs {
    /// The comments associated with the file.
    pub comments: OsString,
    /// The name of the company that produced the file.
    pub company_name: OsString,
    /// The description of the file.
    pub file_description: OsString,
    /// The file version number.
    pub file_version: OsString,
    /// The internal name of the file, if one exists.
    pub internal_name: OsString,
    /// The copyright notices that apply to the specified file.
    pub legal_copyright: OsString,
    /// The trademarks and registered trademarks that apply to the file.
    pub legal_trademarks: OsString,
    /// The name the file was created with.
    pub original_filename: OsString,
    /// The name of the product this file is distributed with.
    pub product_name: OsString,
    /// The version of the product this file is distributed with.
    pub product_version: OsString,
    /// The private build information for the file.
    pub private_build: OsString,
    /// The special build information for the file.
    pub special_build: OsString,
}

impl VersionInfoOs {
    /// Retrieves version information from the specified file.
    /// 
    /// This function is similar to [`VersionInfo::from_file`], but it uses
    /// [`OsString`] instead of [`String`] for its string fields to preserve any
    /// possibly ill-formed UTF-16 data that may be present in the version
    /// information of the file.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if:
    /// - The file does not exist.
    /// - The file is not accessible.
    /// - The version information cannot be retrieved.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use your_crate_name::VersionInfoOs;
    /// 
    /// let info = VersionInfoOs::from_file("path/to/your/file.exe")
    ///    .expect("Failed to retrieve version information");
    /// 
    /// println!("File description: {}", info.file_description.to_string_lossy());
    /// println!("File version: {}", info.file_version.to_string_lossy());
    /// ```
    /// 
    pub fn from_file<P: AsRef<Path>>(file_name: P) -> Result<Self> {
        const LANG_US_ENGLISH_CP_UNKNOWN: u32 = 0x04090000;
        const LANG_US_ENGLISH_CP_UNICODE: u32 = 0x040904B0;
        const LANG_US_ENGLISH_CP_USASCII: u32 = 0x040904E4;
        let ver_data = VersionInfoInternal::from_file(file_name.as_ref())?;
        let ver_info = Self::default();
        Ok(ver_data.get_translation_id()
            .into_iter()
            .chain([
                // anyway, these fallback values are exactly what .NET Framework uses =_=
                LANG_US_ENGLISH_CP_UNICODE,
                LANG_US_ENGLISH_CP_USASCII,
                LANG_US_ENGLISH_CP_UNKNOWN,
            ])
            .map(|translation_id| {
                let mut ver_info = ver_info.clone();
                ver_data.get_all_fields_in_translation(translation_id, &mut ver_info);
                ver_info
            })
            .find(|ver_info| !ver_info.file_version.is_empty())
            .unwrap_or_default())
    }
}

struct VersionInfoInternal(Vec<u8>);
impl VersionInfoInternal {
    fn from_file<P: AsRef<Path>>(file_name: P) -> Result<Self> {
        let file_name = file_name
            .as_ref()
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        let size = unsafe {
            GetFileVersionInfoSizeW(PCWSTR(file_name.as_ptr()), None)
        };
        if size > 0 {
            let mut data = vec![0u8; size as usize];
            unsafe {
                GetFileVersionInfoW(
                    PCWSTR(file_name.as_ptr()),
                    0,
                    size,
                    data.as_mut_ptr().cast())
            }?;
            Ok(Self(data))
        } else {
            Err(Error::from_win32())
        }
    }

    fn get_translation_id(&self) -> Option<u32> {
        self.get_value_by_path(pcwstr!("\\VarFileInfo\\Translation"))
            .filter(|&(_, len)| len >= 4)
            .map(|(ptr, _)| unsafe {
                ptr::read_unaligned::<u32>(ptr.cast())
                    .rotate_right(16)
            })
    }

    fn get_all_fields_in_translation(
        &self,
        translation_id: u32,
        info: &mut VersionInfoOs) {
        info.comments          = self.get_field_in_translation("Comments", translation_id);
        info.company_name      = self.get_field_in_translation("CompanyName", translation_id);
        info.file_description  = self.get_field_in_translation("FileDescription", translation_id);
        info.file_version      = self.get_field_in_translation("FileVersion", translation_id);
        info.internal_name     = self.get_field_in_translation("InternalName", translation_id);
        info.legal_copyright   = self.get_field_in_translation("LegalCopyright", translation_id);
        info.legal_trademarks  = self.get_field_in_translation("LegalTrademarks", translation_id);
        info.original_filename = self.get_field_in_translation("OriginalFilename", translation_id);
        info.product_name      = self.get_field_in_translation("ProductName", translation_id);
        info.product_version   = self.get_field_in_translation("ProductVersion", translation_id);
        info.private_build     = self.get_field_in_translation("PrivateBuild", translation_id);
        info.special_build     = self.get_field_in_translation("SpecialBuild", translation_id);
    }

    fn get_field_in_translation(&self, name: &str, translation_id: u32) -> OsString {
        let path =
            format!("\\StringFileInfo\\{translation_id:08x}\\{name}")
                .encode_utf16()
                .chain(Some(0))
                .collect::<Vec<_>>();
        self.get_value_by_path(PCWSTR(path.as_ptr()))
            .map(|(ptr, len)| OsString::from_wide({
                let mut slice = unsafe {
                    slice::from_raw_parts(ptr.cast(), len)
                };
                while slice.last() == Some(&0) {
                    slice = &slice[..slice.len() - 1];
                }
                slice
            }))
            .unwrap_or_default()
    }

    fn get_value_by_path(&self, path: PCWSTR)
    -> Option<(*const ffi::c_void, usize)> {
        let mut ptr = ptr::null_mut();
        let mut len = 0;
        unsafe {
            VerQueryValueW(
                self.0.as_ptr().cast(),
                PCWSTR(path.as_ptr()),
                &mut ptr,
                &mut len)
        }   .as_bool()
            .then(|| (ptr.cast_const(), len as usize))
    }
}
