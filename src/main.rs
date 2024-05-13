#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;
use std::char::decode_utf16;
use std::collections::HashMap;
use std::ops::{DerefMut, Deref, Not};
use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::io;
#[derive(Clone)]
enum missingObj {
    Spline(String),
    Object(String)

}

fn processMap(map: &str, objVec: &mut Vec<String>) -> Result<(), io::Error> {
    let mappath = Path::new(map);
    let mut written: HashMap<String, ()> = HashMap::new();
    if mappath.exists() {
        let readdir = fs::read_dir(mappath)?;
        for entry in readdir {
            let entry = entry.unwrap(); // without this error that file has been moved in previous loop iter
            //objVec.lock().unwrap().push(String::from(entry.file_name().into_string().unwrap()));
            if (entry.file_type().unwrap().is_file() && entry.file_name().into_string().unwrap().ends_with(".map")) {
                
                if let Ok(file) = fs::File::open(entry.path()) {
                    let strr = utf16_reader::read_to_string(file);
                    let splitstr: Vec<&str> = strr.split("\r\n").collect();
                    for (i, split) in splitstr.iter().enumerate() {
                        let split = *split;
                        if split == "[object]" || split == "[spline]" || split == "[spline_h]" || split == "[splineAttachement]" || split == "[attachObj]" {
                            let thething = splitstr[i + 2].to_string();
                            let mut mappath = PathBuf::from(mappath);
                            mappath.pop();
                            mappath.pop();
                            for pathpart in thething.split("\\") {
                                mappath.push(pathpart);
                            }
                            let newpath = (*mappath.as_path()).to_str().unwrap();
                            //println!("{newpath}");
                            if !written.contains_key(&thething) && !mappath.exists() { // horribly ~in~ Efficient but i will fix it one day
                                //println!("Found missing {thething}");
                                objVec.push(split.to_ascii_uppercase() + " " + thething.clone().as_str() + " " + entry.file_name().to_str().unwrap());
                                written.insert(thething, ());
                            }
                        }
                    }
                } else if let Err(e) = fs::read_to_string(entry.path()) {
                    return Err(e);
                }

            }
        }
    }
    else {
        objVec.push(String::from("Incorrect path"));
    }
    Ok(())
}





fn main() -> Result<(), eframe::Error> {
    let opts = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(500.0, 500.0)),
        ..Default::default()
    };
    
  

    let mut mapFolder = String::new();
    let mut missingObjs: Vec<String> = Vec::new();
    let mut thread: Option<JoinHandle<Vec<String>>> = None;

    eframe::run_simple_native("OMSI 2 Map Integrity Checker", opts, move |ctx, frame| {
        
        
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.heading("OMSI 2 map integrity checker");
            ui.separator();
            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                let labl = ui.label("Pick a map folder");
                ui.text_edit_singleline( &mut mapFolder).labelled_by(labl.id);
                if  ui.button("Open...").clicked() {
                    if let Some(file) = rfd::FileDialog::new().pick_folder() {
                       // let newstr = ;
                        mapFolder.replace_range(.., file.as_os_str().to_str().unwrap_or("Error occured while attempting to get folder path"))  ;
                    }
                }
                
            });
            //let startbtn = ui.button(btnText[isWorking as usize]);
            
            if ui.add_enabled((thread.is_none() || thread.as_ref().unwrap().is_finished()), egui::Button::new("Start")).clicked() {
                //let loadbool = isWorking.load(order) ;
                
                //missingObjs.lock().unwrap().deref_mut().clear();
                thread = Some(thread::spawn({
                    let mut missingObjs: Vec<String> = Vec::new(); 
                    let mapFolder = mapFolder.clone();
                    //let isWorking = isWorking.clone();
                    
                    move || {
                        processMap(&mapFolder, &mut missingObjs);
                        
                        //*isWorking.get_mut() = false;
                        //isWorking.store(false, Ordering::Relaxed);
                        missingObjs
                    }
                })); 
            }

            ui.label("Missing objects:");
            egui::ScrollArea::vertical().min_scrolled_height(0.0).show(ui, |ui| {
                ui.label("test");
                if thread.is_some() && thread.as_ref().unwrap().is_finished() {
                    missingObjs = thread.take().unwrap().join().unwrap();
                    //println!("{}", missingObjs.len());
                }
                for missing in missingObjs.iter() {
                    ui.label(missing);
                }
                
                /*let mut lock = missingObjs.try_lock();
                if let Ok(mutex) = lock {
                    for missing in mutex.deref().iter() {
                        ui.label(missing);
                    }
                } else {
                    let the_error = lock.unwrap_err();
                    if  std::mem::discriminant(&the_error) == std::mem::discriminant(&std::sync::TryLockError::WouldBlock) {
                        let a = the_error.to_string();
                        panic!("{a}");
                    }
                }*/
            });
            
        });
        
    })
}
