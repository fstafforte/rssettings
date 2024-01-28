#  **RSSETTINGS**
This is my first rust crate. It is a library that can be used to manage a clasical  .ini style settings files
[SECTION]
key = value

i.g 
[GLOBAL]
enabled = true

[see documentation](https://docs.rs/rssettings) and the [tests](https://github.com/fstafforte/rssettings) as examples

this software is under [Apache-2.0 license](https://www.apache.org/licenses/LICENSE-2.0)

Please use the devel branch to modify this crate and when your modification has been tested merge them in the master branch

# New Features
27 Jan 2024: Introduced two new methods
1. Settings::section_exists(&self, section_name: &str) -> bool
As the name of the method itself says, it returns a boolean value that indicates whether the section exists or not
2. Settings::key_exists(&self, section_name: &str, key: &str) -> bool
As the name of the method itself says, it returns a boolean value that indicates whether the key exists or not