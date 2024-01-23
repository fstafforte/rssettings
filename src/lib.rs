use std::result::Result as StdResult;
use std::io::{Result as IoResult, Write};
use std::io::{self, BufRead};
use std::fs::File;
use std::fmt::Display;
use std::path::Path;
use std::str::FromStr;
use std::fmt::Debug;



const COMMENT_TAG: &str = "#";
const START_SECTION_TAG: &str = "[";
const END_SECTION_TAG: &str = "]";
const ASSIGN_TAG: &str = "=";
/// GLOBAL_SECTION is a constant that can
/// be used to retrieve all values that do not have a section
/// due to the fact that when loading the settings file
/// a section name is empty or some lines of key/value pairs
/// were found before the first valid section
pub const GLOBAL_SECTION: &str = "GLOBAL";

// A crate privite structure that represents the key/value pair
// inside a Section structure
// line_cnt represent the file line where the key & value 
// has been found during settings file loading (see Settings::load_private)
struct KeyValuePair {
    key: String,
    value: String,
    line_cnt: usize,
}

// Display trait implementation for KetValuePair struct
impl Display for KeyValuePair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "key: {}, value: {}\n", self.key, self.value)
    }
}

// PartialEq trait implementation for KetValuePair struct
impl PartialEq for KeyValuePair {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

// KeyValuePair implementation
impl KeyValuePair {
    // Associated function to create a new KeyValuePair taking ownership of passed arguments
    fn new(key: String, value: String, line_cnt: usize) -> Self {
        Self {
            key,
            value, 
            line_cnt
        }
    }
}



// Private crate struct that represents settings file section
struct Section {
    name: String,
    values: Vec<KeyValuePair>
}

// Display trait implementation for Section structure
impl Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]\n", self.name)?;
        let mut iter = self.values.iter();
        while let Some(key_value) = iter.next() {
           write!(f, "{}", key_value)? 
        }
        write!(f, "\n")
    }    
}

// PartialEq trait implementation for Section structure
impl PartialEq for Section {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.values == other.values
    }
}

// Section struction implementation
impl Section {
    // Associated function to create e new Section
    // [name]: Section name
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            values: vec![],
        }
    }

    // Returns A std::result::Result<(), usize>
    // if key already exists Result::Err contains
    // the previous line where the duplicated
    // key has been found
    // [&self]: Section constant reference
    // [key]: Key name
    // [vaue]: Value associated to the [key]
    // [line_cnt]: Settings file line where the key/value pair has been previously found 
    fn add(&mut self, key: String, value: String, line_cnt: usize) -> StdResult<(), usize> {
        let mut iter = self.values.iter_mut();
        while let Some(key_value) = iter.next() {
            if key_value.key == key {
                return StdResult::Err(key_value.line_cnt.clone());
            }
        }
        self.values.push(KeyValuePair::new(key, value, line_cnt));
        StdResult::Ok(())
    }

    // Resturns if [key] found a reference to key's associated value else Option::None
    // [&self]: Section constant reference
    // [key]: key name
    fn get(&self, key: &str) -> Option<&String> {
        let mut iter = self.values.iter();
        while let Some(key_value) = iter.next() {
            if key_value.key == key {
                return Some(&key_value.value);
            }
        }
        None
    }

    // Sets the new associated value of [key]
    // Returns true if [key] has been found 
    // false otherwise.   
    // [&mut self]: Section mutable reference
    // [key]: key name
    // [value]:: new key's associated value
    fn set(&mut self, key: &str, value: String) -> bool {
        let mut iter = self.values.iter_mut();
        while let Some(key_value) = iter.next() {
            if key_value.key == key {
                key_value.value = value;
                return true;
            }
        }
        false
    } 

    fn unload(&mut self) {
        self.values.clear();
    }
}



const OPENING_FILE_ERROR_MESSAGE_IDX: usize = 0usize;
const MISSING_START_SECTION_TAG_MESSAGE_IDX: usize = OPENING_FILE_ERROR_MESSAGE_IDX + 1usize;
const MISSING_END_SECTION_TAG_MESSAGE_IDX: usize = MISSING_START_SECTION_TAG_MESSAGE_IDX + 1usize;
const MISSING_ASSIGN_TAG_MESSAGE_IDX: usize = MISSING_END_SECTION_TAG_MESSAGE_IDX + 1usize;
const MISSING_KEY_MESSAGE_IDX: usize = MISSING_ASSIGN_TAG_MESSAGE_IDX + 1usize;
const DUPLICATED_KEY_MESSAGE_IDX: usize = MISSING_KEY_MESSAGE_IDX + 1usize;
const SECTION_NOT_FOUND_MESSAGE_IDX: usize = DUPLICATED_KEY_MESSAGE_IDX + 1usize;
const KEY_NOT_FOUND_MESSAGE_IDX: usize = SECTION_NOT_FOUND_MESSAGE_IDX + 1usize;
const PARSING_ERROR_MESSAGE_IDX: usize = KEY_NOT_FOUND_MESSAGE_IDX + 1usize;
const WRITING_FILE_ERROR_MESSAGE_IDX: usize = PARSING_ERROR_MESSAGE_IDX + 1usize;
const READING_FILE_ERROR_MESSAGE_IDX: usize = WRITING_FILE_ERROR_MESSAGE_IDX + 1usize;
const ALREADY_INITIALIZED_MESSAGE_IDX: usize = READING_FILE_ERROR_MESSAGE_IDX + 1usize;
// constant representing the number of errors that rssettings crate can return
pub const MESSAGES_NUMBER: usize = ALREADY_INITIALIZED_MESSAGE_IDX + 1usize;

// Table of default english language errors
const SETTINGS_MESSAGES: [&str; MESSAGES_NUMBER] = [
    "Error opening settings file: '{}': '{}'",
    "Missing start section tag '{}' at line '{}' of settings file: '{}'",
    "Missing end section tag '{}' at line '{}' of settings file: '{}'",
    "Missing assign tag '{}' at line '{}' of settings file: '{}'",
    "Missing key at line '{}' of settings file: '{}'",
    "Duplicated key '{}' at line '{}' previously defined at line '{}' of settings file: '{}'",
    "Section '{}' not found",
    "Section '{}' key '{}' not found",
    "Section '{}' key '{}', Parsing error: '{}'",
    "Error writing file: '{}': '{}'",
    "Error reading file: '{}' at line {}: '{}'",
    "Settings already initialized using file: '{}'"
];


/// Settings::get method returns this structure.
/// It is composed by 2 public attributes 
/// first 'value' is the value returned 
/// second 'error' can contains the possible error occured 
/// during the Setting::get method or an empty string in case everything 
/// has gone well.
/// see Setting get method for an example
pub struct SettingsValue<T> {
    pub value: T,
    pub error: String
}

// Crate privite enumertion
// use to identified the line contained in a settings file
enum LineType {
    EmptyLine,  // Empty line
    SectionLine(String), // Line containing a Section (i.g. [GLOBAL])
    KeyAndValue(String, String), // Line containing a key value pair, value could be an empty string
    BadFormattedLine(String) // Bad formatted line the String retirned is the relative error message
}



/// Setting structure
/// It is composed by 3 private attributes 
/// 'path' contains the path of the loaded settings file 
/// 'sections' is a vector containing Section structures inside the settings file
/// 'messages_table' is a vector of strings representing all error generated by Settings
pub struct Settings {
    path: String,
    sections: Vec<Section>,
    messages_table: Vec<String>
}



impl Settings {
    /// Associated function to create a Settings structure 
    /// This function initialize the messages table with the default english language
    /// Returns an empty Setting structure
    /// 
    /// # Examples
    /// ```
    /// use rssettings::Settings;
    /// 
    /// fn main() {
    ///     let mut settings = Settings::new();
    /// }
    /// ```
    /// 
    pub fn new() -> Self {
        let mut settings = Self {
            path: String::from(""),
            sections: vec![],
            messages_table: vec![]
        };
        for message in SETTINGS_MESSAGES {
            settings.messages_table.push(message.to_string());
        }
        settings
    }


    /// Associated function to create a Settings structure 
    /// This function initialize the messages table with the the user language messages
    /// 'settings_messages' messages vector passed has reference
    /// Returns an empty Setting structure
    /// # Examples
    /// ```
    /// use rssettings::{Settings,MESSAGES_NUMBER};
    /// const IT_SETTINGS_MESSAGES: [&str; MESSAGES_NUMBER] = [
    ///     "Errore apertura file di settings: '{}': '{}'",
    ///     "Manca il tag di inizio sezione '{}' alla linea '{}' del file di settings: '{}'",
    ///     "Manca il tag di fine sezione '{}' alla linea '{}' del file di settings: '{}'",
    ///     "Manca il tag di assegnazione '{}' alla linea '{}' del file di settings: '{}'",
    ///     "Manca la chiave alla linea '{}' del file di settings: '{}'",
    ///     "Chiave duplicata '{}' alla linea '{}' precedentemente definita alla linea '{}' del file di settings: '{}'",
    ///     "Sezione '{}' non trovata",
    ///     "Sezione '{}' chiave '{}' non trovata",
    ///     "Sezione '{}' chiave '{}', Errore di analisi: '{}'",
    ///     "Errore scrittura file: '{}': '{}'",
    ///     "Errore lettura file: '{}' alla line {}: '{}'",
    ///     "Settings già inizializzato utilizzando il file: '{}'"
    /// ];
    /// fn main()  {
    ///     let mut settings = Settings::new_locale_messages(&IT_SETTINGS_MESSAGES);
    ///     match settings.load("settings.ini") {
    ///         Result::Ok(()) => {
    ///         },
    ///         Result::Err(error) => {
    ///             eprintln!("{}", error);
    ///         }
    ///     }
    /// }
    /// ```
    /// 
    pub fn new_locale_messages(settings_messages: &[&str; MESSAGES_NUMBER]) -> Self {
        let mut settings = Self {
            path: String::from(""),
            sections: vec![],
            messages_table: vec![]
        };
        for message in *settings_messages {
            settings.messages_table.push(message.to_string());
        }
        settings

    }

    /// Load a settings file 
    /// Returns std::result::Result::Ok(()) if settings file correctly loaded
    /// or std::result::Result::Err(error: String) if something has gone wrong
    /// error contains a message (in english or user language according to 
    /// user created Setting, Settings::new or Settings::new_locale_messages) 
    /// describing the problem
    /// 
    /// # Examples
    /// ```
    /// use rssettings::Settings;
    /// 
    /// fn main() {
    ///     let mut settings = Settings::new();
    ///     match settings.load("test_files/settings.ini") {
    ///         Result::Ok(()) => {
    ///         },
    ///         Result::Err(error) => {
    ///             eprintln!("{}", error);
    ///         }
    ///     }
    /// }
    /// ```
    /// 
    /// [&mut self] Settings mutable reference
    /// ['path'] settings file path AsRef of std::path::Path
    pub fn load<P>(&mut self, path: P) -> StdResult<(), String> where P: AsRef<Path> {
        if self.is_initialize() {
            return StdResult::Err(
                self.format_message(ALREADY_INITIALIZED_MESSAGE_IDX, vec![&self.path]));
        }

        let result = self.load_private(path);
        if StdResult::Ok(()) != result {
            self.unload();
        }
        result
    }


    /// Save Settings in the file used to load it
    /// User can save Settings every time it changes one of its section/key_value pair
    /// or let the Settings save itself when it is dropped
    /// see trait Drop implementation
    /// Returns std::result::Result::Ok(()) when saving is successfuly done
    /// or std::result::Result::Err(error: String) when a problem occured
    /// error message language depends on new associated function used to crete Settings 
    /// # Examples
    /// ```
    /// use rssettings::Settings;
    /// 
    /// fn main() {
    ///     let mut settings = Settings::new();
    ///     match settings.load("test_files/settings.ini") {
    ///         Result::Ok(()) => {
    ///             if let Result::Ok(()) = settings.set("GLOBAL", "bool_value", true) {
    ///                 if let Result::Err(error) = settings.save() {
    ///                     eprintln!("{}", error);
    ///                 }
    ///             }
    ///         },
    ///         Result::Err(error) => {
    ///             eprintln!("{}", error);
    ///         }
    ///     }
    /// }
    /// ```
    /// 
    /// ['&self'] Settings immutable reference
    pub fn save(&self) -> StdResult<(), String> {
        let mut line_texts: Vec<String> = vec![];
        if self.is_initialize() {
            match File::open(&self.path) {
                IoResult::Ok(settings_file) => {
                    let lines = io::BufReader::new(settings_file).lines();
                    let mut line_cnt = 1usize;
                    for line in lines {
                        match line {
                            IoResult::Ok(line_text) => {
                                line_texts.push(line_text);
                            },
                            IoResult::Err(ioerror) => {
                                let error = format!("{:#}", ioerror);
                                let line = format!("{}", line_cnt);
                                return StdResult::Err(self.format_message(READING_FILE_ERROR_MESSAGE_IDX, 
                                    vec![&self.path, &line, &error]));            
                                }           
                        }
                        line_cnt = line_cnt + 1;
                    }
                },
                IoResult::Err(ioerror) => {
                    let error = format!("{:#}", ioerror);
                    return StdResult::Err(self.format_message(OPENING_FILE_ERROR_MESSAGE_IDX,
                        vec![&self.path, &error]));
                }
            }
            let mut sections_iter = self.sections.iter();
            while let Some(section) = sections_iter.next() {
                let mut values_iter = section.values.iter();
                while let Some(key_value) = values_iter.next() {
                    if let Some(index) = line_texts[key_value.line_cnt - 1].find(COMMENT_TAG) {
                        let comment = &line_texts[key_value.line_cnt - 1][index..];
                        line_texts[key_value.line_cnt - 1] = format!("{} {} {} {}", key_value.key, ASSIGN_TAG, key_value.value, comment);
                    } else {
                        line_texts[key_value.line_cnt - 1] = format!("{} {} {}", key_value.key, ASSIGN_TAG, key_value.value);
                    }
                }
            }
    
            match File::create(&self.path) {
                IoResult::Ok(mut settings_file) => {
                    for line_text in line_texts {
                        if let IoResult::Err(ioerror) = settings_file.write_all(format!("{}\n", line_text).as_bytes()) {
                            let error = format!("{:#}", ioerror);
                            return StdResult::Err(self.format_message(WRITING_FILE_ERROR_MESSAGE_IDX,
                                vec![&self.path, &error]));
                        } else {
                            if let IoResult::Err(ioerror) = settings_file.flush() {
                                let error = format!("{:#}", ioerror);
                                return StdResult::Err(self.format_message(WRITING_FILE_ERROR_MESSAGE_IDX,
                                    vec![&self.path, &error]));
                            }
                        }
                    }
                },
                IoResult::Err(ioerror) => {
                    let error = format!("{:#}", ioerror);
                    return StdResult::Err(self.format_message(OPENING_FILE_ERROR_MESSAGE_IDX,
                        vec![&self.path, &error]));
                }
            }
        }
        StdResult::Ok(())
    }


    /// Generic method use to get section/key value
    /// Generic type parameter has to implement FromStr & Display traits
    /// Returns a SettingsValue structure containing the value associated with the section
    /// and the key if both exist or a default value if not ad SettingsValue.error set with the 
    /// relative error message  
    /// error message language depends on new associated function used to crete Settings 
    /// # Examples
    /// ```
    /// use rssettings::Settings;
    /// 
    /// fn main() {
    ///     let mut settings = Settings::new();
    ///     match settings.load("test_files/settings.ini") {
    ///         Result::Ok(()) => {
    ///             let bool_value = settings.get("GLOBAL", "bool_value", false);
    ///             if bool_value.error.len() == 0 {
    ///                 assert_eq!(true, bool_value.value);
    ///             } else {
    ///                 eprintln!("{}", bool_value.error);
    ///             }
    ///         },
    ///         Result::Err(error) => {
    ///             eprintln!("{}", error);
    ///         }
    ///     }
    /// }
    /// ```
    /// 
    /// ['&self'] Settings immutable reference
    /// [&str section_name] section name
    /// [&str key] key name
    /// [T default_value] default value in case of error
    pub fn get<T: FromStr + Display>(&self, section_name: &str, key: &str, default_value: T) -> SettingsValue<T> where <T as FromStr>::Err: Debug {
        let mut result = SettingsValue {value: default_value, error: String::from("")};

        if let Some(section) = self.get_section(section_name) {
            if let Some(value) = section.get(key) {
                match value.parse::<T>() {
                    StdResult::Ok(parsed_value) => {
                        result.value = parsed_value;
                    },
                    StdResult::Err(error) => {
                        let error = format!("{:#?}", error);
                        let sname = section_name.to_string();
                        let kname = key.to_string();
                        result.error = self.format_message(PARSING_ERROR_MESSAGE_IDX, 
                            vec![&sname, &kname, &error]);
                    }
                }
            } else {
                let sname = section_name.to_string();
                let kname = key.to_string();
                result.error = self.format_message(KEY_NOT_FOUND_MESSAGE_IDX, 
                    vec![&sname, &kname]);
            }
        } else {
            let sname = section_name.to_string();
            result.error = self.format_message(SECTION_NOT_FOUND_MESSAGE_IDX, 
                vec![&sname]);
        }

        result
    }


    /// # Examples
    /// ```
    /// use rssettings::Settings;
    /// 
    /// fn main() {
    ///     let mut settings = Settings::new();
    ///     match settings.load("test_files/settings.ini") {
    ///         Result::Ok(()) => {
    ///             let orig_bool_value = settings.get("GLOBAL", "bool_value", false);
    ///             if orig_bool_value.error.len() == 0 {
    ///                 settings.set("GLOBAL", "bool_value", false).unwrap_or_else(|error| {
    ///                     eprintln!("{}", error);
    ///                 });
    ///                 let new_bool_value = settings.get("GLOBAL", "bool_value", true);
    ///                 assert_ne!(orig_bool_value.value, new_bool_value.value);
    ///                 settings.set("GLOBAL", "bool_value", true).unwrap_or_else(|error| {
    ///                     eprintln!("{}", error);
    ///                 });
    ///             } else {
    ///                 eprintln!("{}", orig_bool_value.error);
    ///             }
    ///         },
    ///         Result::Err(error) => {
    ///             eprintln!("{}", error);
    ///         }
    ///     }
    /// }
    /// ```
    /// 
    pub fn set<T: Display>(&mut self, section_name: &str, key: &str, value: T) -> StdResult<(), String> {
        if let Some(section) = self.get_section_mut(section_name) {
            if !section.set(key, value.to_string()) {
                let sname = section_name.to_string();
                let kname = key.to_string();
                return StdResult::Err(self.format_message(KEY_NOT_FOUND_MESSAGE_IDX, 
                    vec![&sname, &kname]));
            }
            StdResult::Ok(())
        } else {
            let sname = section_name.to_string();
            return StdResult::Err(self.format_message(SECTION_NOT_FOUND_MESSAGE_IDX, 
                vec![&sname]));
        }
    }

    // Private methods & functions

    // This method is in charge to load the file passed to the public method load
    // Returns std::result::Result::Ok(()) in case file has been succesufuly loaded 
    // otherwhise std::result::Result::Err(erroe: String) error contains the reason why 
    // file has not been loaded
    // [&mut self] Settings mutable reference
    // [path] settings file path AsRef of std::path::Path 
    fn load_private<P>(&mut self, path: P) -> StdResult<(), String> where P: AsRef<Path> {


        let path_str = path.as_ref().as_os_str().to_str().unwrap_or("");
        match File::open(path_str) {
            IoResult::Ok(settings_file) => {
                let lines = io::BufReader::new(settings_file).lines();
                let mut line_cnt = 1usize;
                let mut current_section = String::from(GLOBAL_SECTION);
                for line in lines {
                    match line {
                        IoResult::Ok(line_text) => {
                            match self.line_type(&line_text, &line_cnt, path_str) {
                                LineType::SectionLine(section_name) => {
                                    if current_section != section_name {
                                        current_section = section_name;
                                    }
                                },
                                LineType::KeyAndValue(key, value) => {
                                    self.add_to_section(&current_section, key, value, line_cnt.clone(), path_str)?;
                                },
                                LineType::BadFormattedLine(error) => {
                                    return StdResult::Err(error);
                                },
                                LineType::EmptyLine => {

                                }
                            }
                        },
                        IoResult::Err(ioerror) => {
                            let error = format!("{:#}", ioerror);
                            let line = format!("{}", line_cnt);
                            return StdResult::Err(self.format_message(READING_FILE_ERROR_MESSAGE_IDX, 
                                vec![&path_str.to_string(), &line, &error]));            
                        }
                    }
                    line_cnt = line_cnt + 1;
                }
            },
            IoResult::Err(ioerror) => {
                let error = format!("{:#}", ioerror);
                return StdResult::Err(self.format_message(OPENING_FILE_ERROR_MESSAGE_IDX,
                    vec![&path_str.to_string(), &error]));
            }

        }

        self.path = path_str.to_string();
        StdResult::Ok(())
    }

    // This method is privatly used to clean Setting stucture content
    // it is used when the load methos fails
    // [&mut self] Settings mutable reference
    fn unload(&mut self) {
        let mut iter = self.sections.iter_mut();
        while let Some(section) = iter.next() {
            section.unload();
        }
        self.sections.clear();
    }


    // This method format the error message string in base to the message index
    // and parameters passed as a vector of string reference
    // Returns the formatted error message
    // [&sef] Settings immutable reference
    // [message_idx] Message index 
    // [params] Vector containing parameters as String reference that must be places
    // in the message placeholders '{}'
    // 
    fn format_message(&self, message_idx: usize, params: Vec<&String>) -> String {
        let mut message = self.messages_table[message_idx].clone();
        let mut i = 0usize;
        while let Some(_) = message.find("{}"){
            message = message.replacen("{}", &params[i].to_owned(), 1);
            i = i + 1;
            if i >= params.len() {
                break;
            }
        }

        message
    }

    // Returns if the Setting is already initialized or not
    // The Settings is initialized if the settings file has beee
    // successfuly loaded and so the path has been set
    // [&sef] Settings immutable reference
    //
    fn is_initialize(&self) -> bool {
        0 != self.path.len()
    }

    // Returns the setting file line type, see LineType enumeration
    // [&sef] Settings immutable reference
    // [line_text] Reference to the line text to analyze
    // [line_cnt] Reference to the line counter (to return the error mesage containing the bad line number) 
    // [settings_file] Reference to the settings file path (to return the error mesage containing the settings file path)
    // 
    fn line_type(&self, line_text: &String, line_cnt: &usize, settings_file: &str) -> LineType {
        let mut trimmed_line = line_text.clone();
        if let Some(index) = trimmed_line.find(COMMENT_TAG) {
            trimmed_line.truncate(index)
        }


        let trimmed_line = trimmed_line.trim();
        if 0 == trimmed_line.len() {
            return LineType::EmptyLine;
        }
        let starts_with = trimmed_line.starts_with(START_SECTION_TAG);
        let ends_with = trimmed_line.ends_with(END_SECTION_TAG);

        if starts_with && ends_with {
            let mut section_name = trimmed_line[1..trimmed_line.len() - 1].to_string();
            if 0 == section_name.len() {
                section_name = String::from(GLOBAL_SECTION);
            } 
            return LineType::SectionLine(section_name);
        } else if starts_with && !ends_with {
            let tag = END_SECTION_TAG.to_string();
            let line = format!("{}", line_cnt);
            let path = settings_file.to_string();
            let error = self.format_message(MISSING_END_SECTION_TAG_MESSAGE_IDX, 
                vec![&tag, &line, &path]);
            return LineType::BadFormattedLine(error);
        } else if !starts_with && ends_with {
            let tag = START_SECTION_TAG.to_string();
            let line = format!("{}", line_cnt);
            let path = settings_file.to_string();
            let error = self.format_message(MISSING_START_SECTION_TAG_MESSAGE_IDX,  
                vec![&tag, &line, &path]);
            return LineType::BadFormattedLine(error);
        }

        if let Some(assign_pos) = trimmed_line.find(ASSIGN_TAG) {
            let (mut key, mut value) = trimmed_line.split_at(assign_pos);
            key = key.trim();
            let removed_assign = value.replace(ASSIGN_TAG, "");
            value = removed_assign.trim();
            if 0 == key.len() {
                let line = format!("{}", line_cnt);
                let path = settings_file.to_string();
                let error = self.format_message(MISSING_KEY_MESSAGE_IDX,
                    vec![&line, &path]);
                return LineType::BadFormattedLine(error);
            }
            return LineType::KeyAndValue(key.to_string(), value.to_string());
        }

        let tag = ASSIGN_TAG.to_string();
        let line = format!("{}", line_cnt);
        let path = settings_file.to_string();
        let error = self.format_message(MISSING_ASSIGN_TAG_MESSAGE_IDX, 
            vec![&tag, &line, &path]);
        return LineType::BadFormattedLine(error);
    }

    // Adds a key/value pair to a Section
    // [&mut self] Settings mutable reference
    // [section_name] Section name reference
    // [key] Key name 
    // [value] Value relative to the key
    // [line_cnt] Settings file key/value pair Line number 
    // [settings_file] Settibg file path reference
    fn add_to_section(&mut self, section_name: &String, key: String, value: String, line_cnt: usize, settings_file: &str) -> StdResult<(), String> {
        let mut iter = self.sections.iter_mut();
        while let Some(section) = iter.next() {
            if section.name == *section_name {
                let kname = key.clone();
                if let StdResult::Err(previous_line) = section.add(key, value, line_cnt) {
                    let line = format!("{}", line_cnt);
                    let previous_line = format!("{}", previous_line);
                    let path = settings_file.to_string();               
                    let error = self.format_message(DUPLICATED_KEY_MESSAGE_IDX, 
                        vec![&kname, &line, &previous_line, &path]);
                    return StdResult::Err(error);
                }
                return StdResult::Ok(());
            }
        }
        let mut section = Section::new(&section_name);
        let _ = section.add(key, value, line_cnt);
        self.sections.push(section);
        StdResult::Ok(())
    }

    // Returns a core::option::Option::Some() containing an immutable reference to Section
    // if the searched section name exists, None if not
    // [&self] Settings immutable reference
    // [section_name] Section name reference
    //
    fn get_section(&self, section_name: &str) -> Option<&Section> {
        let mut iter = self.sections.iter();
        while let Some(section) = iter.next() {
            if section.name == section_name {
                return Some(section);
            }
        }
        None
    }

    // Returns a core::option::Option::Some() containing an mutable reference to Section
    // if the searched section name exists, None if not
    // [&mut self] Settings mutable reference
    // [section_name] Section name reference
    //
    fn get_section_mut(&mut self, section_name: &str) -> Option<&mut Section> {
        let mut iter = self.sections.iter_mut();
        while let Some(section) = iter.next() {
            if section.name == section_name {
                return Some(section);
            }
        }
        None
    }
}

/// implementation of Display trait for the Settings structure
/// # Examples
/// ```
/// use rssettings::Settings;
/// 
/// fn main() {
///     let mut settings = Settings::new();
///     settings.load("test_files/settings");
///     println!("{}", settings);
/// }
/// 
/// ```
impl Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Settings path: {}\n", self.path)?;
        let mut iter = self.sections.iter();
        while let Some(section) = iter.next() {
            write!(f, "{}", section)?;
        }
        write!(f, "====================================================================\n") 
    }
}

// implementation of Drop trait for the Settings structure
impl Drop for Settings {
    fn drop(&mut self) {
        if let StdResult::Err(error) = self.save() {
            eprint!("'{}': {:#?}", self.path, error);
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::{self, Builder};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn load_errors() {
        let mut settings_file_path ="goofy.ini";
        let mut settings = Settings::new();
        assert_ne!(Result::Ok(()), settings.load(settings_file_path));
        settings_file_path ="test_files";
        assert_ne!(Result::Ok(()), settings.load(settings_file_path));
    }
    #[test]
    fn missing_start_section_tag() {
        let settings_file_path ="test_files/missing_start_section_tag.ini";
        let mut settings = Settings::new();
        let error = StdResult::Err(
            format!("Missing start section tag '{}' at line '{}' of settings file: '{}'",
             START_SECTION_TAG, 7, settings_file_path));
        assert_eq!(error, settings.load(settings_file_path));
    }

    #[test]
    fn missing_end_section_tag() {
        let settings_file_path ="test_files/missing_end_section_tag.ini";
        let mut settings = Settings::new();
        let error = StdResult::Err(
            format!("Missing end section tag '{}' at line '{}' of settings file: '{}'",
             END_SECTION_TAG, 11, settings_file_path));
        assert_eq!(error, settings.load(settings_file_path));
    }
    #[test]
    fn missing_assign_tag() {
        let settings_file_path ="test_files/missing_assign_tag.ini";
        let mut settings = Settings::new();
        let error = StdResult::Err(
            format!("Missing assign tag '{}' at line '3' of settings file: '{}'", ASSIGN_TAG, settings_file_path));
        assert_eq!(error, settings.load(settings_file_path));
    }

    #[test]
    fn missing_key() {
        let settings_file_path ="test_files/missing_key.ini";
        let mut settings = Settings::new();
        let error = StdResult::Err(
            format!("Missing key at line '5' of settings file: '{}'", settings_file_path)
        );
        assert_eq!(error, settings.load(settings_file_path));
    }

    #[test]
    fn duplicated_key() {
        let settings_file_path ="test_files/duplicated_key.ini";
        let mut settings = Settings::new();
        let error = StdResult::Err(
            format!("Duplicated key 'key1' at line '5' previously defined at line '2' of settings file: '{}'", settings_file_path)
            );
        assert_eq!(error, settings.load(settings_file_path));
    }

    #[test]
    fn key_value_to_global() {
        let settings_file_path ="test_files/key_value_to_global.ini";
        let mut settings = Settings::new();
        assert_eq!(Result::Ok(()), settings.load(settings_file_path));

        let settings_dump = 
"Settings path: test_files/key_value_to_global.ini
[GLOBAL]
key: key1, value: true
key: key2, value: 123
key: key3, value: 234.35
key: key4, value: abc

[SECTION_1]
key: key1, value: def

====================================================================
".to_string();

        assert_eq!(settings_dump, format!("{}", settings));
    }

   #[test]
    fn set_get_errors() {
        let settings_file_path ="test_files/set_get_errors.ini";
        let mut settings = Settings::new();
        assert_eq!(Result::Ok(()), settings.load(settings_file_path));
        assert_eq!("Section 'GENERLA' not found".to_string(), settings.get("GENERLA", "enabled", false).error);
        assert_eq!("Section 'GENERAL' key 'enable' not found".to_string(), settings.get("GENERAL", "enable", false).error);
        let error = "a123".parse::<i32>().err().unwrap();
        let mut error_as_string = format!("Section 'GENERAL' key 'integer_value', Parsing error: '{:#?}'", error);
        assert_eq!(error_as_string, settings.get("GENERAL", "integer_value", 10).error);

        let error = "123a.35".parse::<f32>().err().unwrap();
        error_as_string = format!("Section 'GENERAL' key 'float_value', Parsing error: '{:#?}'", error);
        assert_eq!(error_as_string, settings.get("GENERAL", "float_value", -1.0f32).error);

        let error = "ciao".parse::<bool>().err().unwrap();
        error_as_string = format!("Section 'GENERAL' key 'enabled', Parsing error: '{:#?}'", error);
        assert_eq!(error_as_string, settings.get("GENERAL", "enabled", true).error);

        let mut error = "Section 'GLOBAL' not found".to_string();
        assert_eq!(StdResult::Err(error), settings.set(GLOBAL_SECTION, "enabled", true));
        error = "Section 'GENERAL' key 'enable' not found".to_string();
        assert_eq!(StdResult::Err(error), settings.set("GENERAL", "enable", false));
    } 

    #[test]
    fn no_section_name() {
        let settings_file_path ="test_files/no_section_name.ini";
        let mut settings = Settings::new();
        assert_eq!(Result::Ok(()), settings.load(settings_file_path));
        let result = settings.get(GLOBAL_SECTION, "title", "???".to_string());
        assert!((result.error.len() == 0 && result.value == "Test empty section name".to_string()));
    }

    #[test]
    fn get_set_ok() {
        let settings_file_path ="test_files/settings.ini";
        let mut settings = Settings::new();
        assert_eq!(Result::Ok(()), settings.load(settings_file_path));

        let mut result = settings.get(GLOBAL_SECTION, "bool_value", false);
        assert!(result.error.len() == 0 && true == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "bool_value", false));
        result = settings.get(GLOBAL_SECTION, "bool_value", true);
        assert!(result.error.len() == 0 && false == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "bool_value", true));
        result = settings.get(GLOBAL_SECTION, "bool_value", false);
        assert!(result.error.len() == 0 && true == result.value);

        let mut result = settings.get(GLOBAL_SECTION, "i32_value", -1000i32);
        assert!(result.error.len() == 0 && -100i32 == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "i32_value", -1000i32));
        result = settings.get(GLOBAL_SECTION, "i32_value", -100i32);
        assert!(result.error.len() == 0 && -1000i32 == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "i32_value", -100i32));
        result = settings.get(GLOBAL_SECTION, "i32_value", -1000i32);
        assert!(result.error.len() == 0 && -100i32 == result.value);

        let mut result = settings.get(GLOBAL_SECTION, "u32_value", 1000u32);
        assert!(result.error.len() == 0 && 100u32 == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "u32_value", 1000u32));
        result = settings.get(GLOBAL_SECTION, "u32_value", 100u32);
        assert!(result.error.len() == 0 && 1000u32 == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "u32_value", 100u32));
        result = settings.get(GLOBAL_SECTION, "u32_value", 1000u32);
        assert!(result.error.len() == 0 && 100u32 == result.value);

        let mut result = settings.get(GLOBAL_SECTION, "i64_value", -2000i64);
        assert!(result.error.len() == 0 && -200i64 == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "i64_value", -2000i64));
        result = settings.get(GLOBAL_SECTION, "i64_value", -200i64);
        assert!(result.error.len() == 0 && -2000i64 == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "i64_value", -200i64));
        result = settings.get(GLOBAL_SECTION, "i64_value", -2000i64);
        assert!(result.error.len() == 0 && -200i64 == result.value);

        let mut result = settings.get(GLOBAL_SECTION, "u64_value", 2000u64);
        assert!(result.error.len() == 0 && 200u64 == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "u64_value", 2000u64));
        result = settings.get(GLOBAL_SECTION, "u64_value", 200u64);
        assert!(result.error.len() == 0 && 2000u64 == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "u64_value", 200u64));
        result = settings.get(GLOBAL_SECTION, "u64_value", 2000u64);
        assert!(result.error.len() == 0 && 200u64 == result.value);

        let mut result = settings.get(GLOBAL_SECTION, "isize_value", -3000isize);
        assert!(result.error.len() == 0 && -300isize == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "isize_value", -3000isize));
        result = settings.get(GLOBAL_SECTION, "isize_value", -300isize);
        assert!(result.error.len() == 0 && -3000isize == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "isize_value", -300isize));
        result = settings.get(GLOBAL_SECTION, "isize_value", -3000isize);
        assert!(result.error.len() == 0 && -300isize == result.value);

        let mut result = settings.get(GLOBAL_SECTION, "usize_value", 3000usize);
        assert!(result.error.len() == 0 && 300usize == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "usize_value", 3000usize));
        result = settings.get(GLOBAL_SECTION, "usize_value", 300usize);
        assert!(result.error.len() == 0 && 3000usize == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "usize_value", 300usize));
        result = settings.get(GLOBAL_SECTION, "usize_value", 3000usize);
        assert!(result.error.len() == 0 && 300usize == result.value);

        let mut result = settings.get(GLOBAL_SECTION, "f32_value", -32.400f32);
        assert!(result.error.len() == 0 && -400.32f32 == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "f32_value", -32.400f32));
        result = settings.get(GLOBAL_SECTION, "f32_value", -400.32f32);
        assert!(result.error.len() == 0 && -32.400f32 == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "f32_value", -400.32f32));
        result = settings.get(GLOBAL_SECTION, "f32_value", -32.400f32);
        assert!(result.error.len() == 0 && -400.32f32 == result.value);

        let mut result = settings.get(GLOBAL_SECTION, "f64_value", 64.400f64);
        assert!(result.error.len() == 0 && 400.64f64 == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "f64_value", 64.400f64));
        result = settings.get(GLOBAL_SECTION, "f64_value", 400.64f64);
        assert!(result.error.len() == 0 && 64.400f64 == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "f64_value", 400.64f64));
        result = settings.get(GLOBAL_SECTION, "f64_value", 64.400f64);
        assert!(result.error.len() == 0 && 400.64f64 == result.value);

        let mut result = settings.get(GLOBAL_SECTION, "string_value", "boh!!!".to_string());
        assert!(result.error.len() == 0 && "The quick brown fox jump over the lazy dog" == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "string_value", "boh!!!".to_string()));
        result = settings.get(GLOBAL_SECTION, "string_value", "???".to_string());
        assert!(result.error.len() == 0 && "boh!!!" == result.value);
        assert!(Result::Ok(()) == settings.set(GLOBAL_SECTION, "string_value", "The quick brown fox jump over the lazy dog".to_string()));
        result = settings.get(GLOBAL_SECTION, "string_value", "boh!!!".to_string());
        assert!(result.error.len() == 0 && "The quick brown fox jump over the lazy dog" == result.value);
        
        let mut original_settings = Settings::new();
        let _ = original_settings.load("test_files/original_settings.ini");

        assert!(settings.sections == original_settings.sections);
    }

    #[test]
    fn settings_with_locale_messages() {
        const IT_SETTINGS_MESSAGES: [&str; MESSAGES_NUMBER] = [
            "Errore apertura file di settings: '{}': '{}'",
            "Manca il tag di inizio sezione '{}' alla linea '{}' del file di settings: '{}'",
            "Manca il tag di fine sezione '{}' alla linea '{}' del file di settings: '{}'",
            "Manca il tag di assegnazione '{}' alla linea '{}' del file di settings: '{}'",
            "Manca la chiave alla linea '{}' del file di settings: '{}'",
            "Chiave duplicata '{}' alla linea '{}' precedentemente definita alla linea '{}' del file di settings: '{}'",
            "Sezione '{}' non trovata",
            "Sezione '{}' chiave '{}' non trovata",
            "Sezione '{}' chiave '{}', Errore di analisi: '{}'",
            "Errore scrittura file: '{}': '{}'",
            "Errore lettura file: '{}' alla line {}: '{}'",
            "Settings già inizializzato utilizzando il file: '{}'"
        ];
        

        let mut settings = Settings::new_locale_messages(&IT_SETTINGS_MESSAGES);
        assert_eq!(Result::Err("Errore apertura file di settings: 'goofy.ini': 'Impossibile trovare il file specificato. (os error 2)'".to_string()), settings.load("goofy.ini"));
        assert_eq!(Result::Err("Manca il tag di inizio sezione '[' alla linea '7' del file di settings: 'test_files/missing_start_section_tag.ini'".to_string()), settings.load("test_files/missing_start_section_tag.ini"));
        assert_eq!(Result::Err("Manca il tag di fine sezione ']' alla linea '11' del file di settings: 'test_files/missing_end_section_tag.ini'".to_string()), settings.load("test_files/missing_end_section_tag.ini"));
        assert_eq!(Result::Err("Manca il tag di assegnazione '=' alla linea '3' del file di settings: 'test_files/missing_assign_tag.ini'".to_string()), settings.load("test_files/missing_assign_tag.ini"));
        assert_eq!(Result::Err("Manca la chiave alla linea '5' del file di settings: 'test_files/missing_key.ini'".to_string()), settings.load("test_files/missing_key.ini"));
        assert_eq!(Result::Err("Chiave duplicata 'key1' alla linea '5' precedentemente definita alla linea '2' del file di settings: 'test_files/duplicated_key.ini'".to_string()), settings.load("test_files/duplicated_key.ini"));
        assert_eq!(Result::Ok(()), settings.load("test_files/set_get_errors.ini"));
        assert_eq!("Sezione 'GENERALE' non trovata".to_string(), settings.get("GENERALE", "enabled", false).error);
        assert_eq!("Sezione 'GENERAL' chiave 'enable' non trovata".to_string(), settings.get("GENERAL", "enable", true).error);
        assert_eq!("Sezione 'GENERAL' chiave 'float_value', Errore di analisi: 'ParseFloatError {\n    kind: Invalid,\n}'".to_string(), settings.get("GENERAL", "float_value", -1.0f32).error);
        assert_eq!(Result::Err("Settings già inizializzato utilizzando il file: 'test_files/set_get_errors.ini'".to_string()), settings.load("test_files/settings.ini"));
        assert_eq!(true , settings.get("LOG", "enabled", false).value);
    }

    #[test]
    fn multi_thread_settings() {
        let settings_file_path ="test_files/settings.ini";
        let mut settings = Settings::new();
        assert_eq!(Result::Ok(()), settings.load(settings_file_path));

        let settings= Arc::new(Mutex::new(settings));
        let thread1_settings = settings.clone(); 
        let builder = Builder::new();
        let thread_handler = builder.name("threa1".to_string()).spawn(move || {
            println!("{} START", thread::current().name().unwrap_or("???"));
            thread::sleep(Duration::from_millis(3000));
            let _ = thread1_settings.lock().unwrap().set(GLOBAL_SECTION, "bool_value", false);
            println!("{} END", thread::current().name().unwrap_or("???"));
        }).unwrap();

        let mut wait = true;
        let mut cnt = 1;
        while wait {
            {
                wait = settings.lock().unwrap().get(GLOBAL_SECTION, "bool_value", true).value;
            }
            thread::sleep(Duration::from_millis(100));
            print!(".");
            std::io::stdout().flush().unwrap_or(());
            if 0 == cnt % 4 {
                println!("");
            }
            cnt = cnt + 1;
        }
        println!("");
        thread_handler.join().unwrap_or(());
        {
            assert!(Result::Ok(()) == settings.lock().unwrap().set(GLOBAL_SECTION, "bool_value", true));
        }
        let result = settings.lock().unwrap().get(GLOBAL_SECTION, "bool_value", false); 
        assert!(result.error.len() == 0 && true == result.value);
    }
}
