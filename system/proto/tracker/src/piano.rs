use std::collections::HashSet;
use std::io::Write;

use core::{Note, Tone};

const KEY_WIDTH: usize = 5;
const BLACK_H: usize = 4;
const WHITE_H: usize = 3;


pub fn render(all_keys: &[Tone], active: &[Tone]) -> Vec<String> {
    let active_set: HashSet<(Note, i32)> =
        active.iter().map(|t| (t.note, t.octave)).collect();

    let whites: Vec<&Tone> = all_keys.iter().filter(|t| t.note.is_white()).collect();
    let blacks: Vec<&Tone> = all_keys.iter().filter(|t| t.note.is_black()).collect();

    let n_whites = whites.len();
    let total_w = n_whites * (KEY_WIDTH - 1) + 1;

    let mut rows: Vec<Vec<char>> =
        vec![vec![' '; total_w]; BLACK_H + WHITE_H + 1];

        
    for wi in 0..=n_whites {
        let col = wi * (KEY_WIDTH - 1);
        if col < total_w {
            for row in rows.iter_mut() {
                row[col] = '|';
            }
        }
    }

    
    for (wi, &wt) in whites.iter().enumerate() {
        let l = wi * (KEY_WIDTH - 1) + 1;
        let r = (wi + 1) * (KEY_WIDTH - 1);
        let fill = if active_set.contains(&(wt.note, wt.octave)) { 'X' } else { ' ' };
        for row in BLACK_H..(BLACK_H + WHITE_H) {
            for c in l..r {
                rows[row][c] = fill;
            }
        }
    }

    
    let bot = BLACK_H + WHITE_H;
    for wi in 0..n_whites {
        let l = wi * (KEY_WIDTH - 1) + 1;
        let r = (wi + 1) * (KEY_WIDTH - 1);
        for c in l..r {
            rows[bot][c] = '_';
        }
    }

    
    for &bt in &blacks {
        let Some(wi) = black_left_white_index(bt, &whites) else { continue };
        let centre = wi * (KEY_WIDTH - 1) + (KEY_WIDTH - 1);
        let fill = if active_set.contains(&(bt.note, bt.octave)) { 'X' } else { ' ' };
        for row in 0..BLACK_H {
            if centre > 0 && centre - 1 < total_w {
                rows[row][centre - 1] = '|';
            }
            if centre < total_w {
                rows[row][centre] = fill;
            }
            if centre + 1 < total_w {
                rows[row][centre + 1] = '|';
            }
        }
    }

    
    let mut label = vec![' '; total_w];
    for (wi, &wt) in whites.iter().enumerate() {
        if wt.note == Note::C {
            let col = wi * (KEY_WIDTH - 1) + 1;
            let s = format!("C{}", wt.octave);
            for (j, ch) in s.chars().enumerate() {
                if col + j < total_w {
                    label[col + j] = ch;
                }
            }
        }
    }

    let mut lines: Vec<String> = rows.iter().map(|r| r.iter().collect()).collect();
    lines.push(label.iter().collect());
    lines
}

/// provides the white-key index of the white key immediately to the left of a black key, like C# -> C
fn black_left_white_index(black: &Tone, whites: &[&Tone]) -> Option<usize> {
    let left_note = match black.note {
        Note::Cs => Note::C,
        Note::Ds => Note::D,
        Note::Fs => Note::F,
        Note::Gs => Note::G,
        Note::As => Note::A,
        _ => return None,
    };
    whites
        .iter()
        .position(|w| w.note == left_note && w.octave == black.octave)
}


pub fn redraw(all_keys: &[Tone], active: &[Tone], prev_height: &mut Option<usize>) {
    let header = if active.is_empty() {
        "(silence)".to_string()
    } else {
        let note_strs: Vec<String> = active.iter().map(|t| t.to_string()).collect();
        format!("playing: {}", note_strs.join("  "))
    };

    let mut lines = vec![header];
    lines.extend(render(all_keys, active));

    match prev_height {
        None => {
            *prev_height = Some(lines.len());
            println!("{}", lines.join("\n"));
        }
        Some(h) => {
            print!("\x1b[{}A", h);

            for line in &lines {
                println!("{:<120}", line);
            }
            
            std::io::stdout().flush().ok();
        }
    }
}