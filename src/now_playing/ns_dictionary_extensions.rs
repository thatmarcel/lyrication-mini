use std::intrinsics::transmute;
use core_foundation::date::CFDateGetAbsoluteTime;
use icrate::Foundation::{NSDate, NSDictionary, NSNumber, NSString};
use objc2::rc::Id;
use objc2::runtime::NSObject;

pub trait NSDictionaryExtensions {
    unsafe fn get_string_for_key(&self, key: &str) -> Option<String>;

    unsafe fn get_f64_for_key(&self, key: &str) -> Option<f64>;

    #[allow(dead_code)]
    unsafe fn get_bool_for_key(&self, key: &str) -> Option<bool>;

    unsafe fn get_absolute_date_time_for_key(&self, key: &str) -> Option<f64>;
}

impl NSDictionaryExtensions for NSDictionary<NSString, NSObject> {
    unsafe fn get_string_for_key(&self, key: &str) -> Option<String> {
        match &self.objectForKey(
            &*NSString::from_str(key)
        ) {
            Some(value) => Option::from(Id::cast::<NSString>(value.to_owned()).to_string()),
            None => None
        }
    }

    unsafe fn get_f64_for_key(&self, key: &str) -> Option<f64> {
        match &self.objectForKey(
            &*NSString::from_str(key)
        ) {
            Some(value) => Option::from(Id::cast::<NSNumber>(value.to_owned()).as_f64()),
            None => None
        }
    }

    unsafe fn get_bool_for_key(&self, key: &str) -> Option<bool> {
        match &self.objectForKey(
            &*NSString::from_str(key)
        ) {
            Some(value) => Option::from(Id::cast::<NSNumber>(value.to_owned()).as_bool()),
            None => None
        }
    }

    unsafe fn get_absolute_date_time_for_key(&self, key: &str) -> Option<f64> {
        match &self.objectForKey(
            &*NSString::from_str(key)
        ) {
            Some(value) => Option::from(CFDateGetAbsoluteTime(transmute(Id::cast::<NSDate>(value.to_owned())))),
            None => None
        }
    }
}