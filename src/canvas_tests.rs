mod constants;

const NUM_CHECKER_ROWS: usize = 8;
const PIXELS_PER_SQUARE: usize = CANVAS_WIDTH / NUM_CHECKER_ROWS;

// Function to generate our checkerboard, pixel by pixel
pub fn generate_checker_board(black_value: u8, white_value: u8) {
  
  // Since Linear memory is a 1 dimensional array, but we want a grid
  // we will be doing 2d to 1d mapping
  // https://softwareengineering.stackexchange.com/questions/212808/treating-a-1d-data-structure-as-2d-grid
  for y in 0..CANVAS_SIZE {
    for x in 0..CANVAS_SIZE {
      // Set our default case to be dark squares
      let mut is_dark_square: bool = true;
      
      // We should change our default case if
      // We are on an odd y
      if (y / PIXELS_PER_SQUARE) % 2 == 0 {
        is_dark_square = false;
      }
      
      // Lastly, alternate on our x value
      if (x / PIXELS_PER_SQUARE) % 2 == 0 {
        is_dark_square = !is_dark_square;
      }
      
      // Now that we determined if we are dark or light,
      // Let's set our square value
      // David: "I'm gonna flex my knowledge of 'option' right now!"
      let mut square_value: Option<u8> = None;
      
      match is_dark_square && square_value == None {
        true => square_value = Some(black_value),
        false => square_value = Some(white_value),
      }
      
      // Let's calculate our index, using our 2d -> 1d mapping.
      // And then multiple by 4, for each pixel property (r,g,b,a).
      let square_number: usize = y * CANVAS_SIZE + x;
      let square_rgba_index: usize = square_number * 4;
      
      // Finally store the values.
      unsafe {
        OUTPUT_BUFFER[square_rgba_index + 0] = square_value.unwrap(); // Red
        OUTPUT_BUFFER[square_rgba_index + 1] = square_value.unwrap(); // Green
        OUTPUT_BUFFER[square_rgba_index + 2] = square_value.unwrap(); // Blue
        OUTPUT_BUFFER[square_rgba_index + 3] = 255; // Alpha (Always Opaque)
      }
    }
  }
}

pub fn clock_anim(seconds:f32) {
    let size:f32 = 120.0;
    let xcoord   = 240.0 + ((seconds * PERCENT_OF_MINUTE).sin() * size);
    let ycoord   = 240.0 + ((seconds * PERCENT_OF_MINUTE).cos() * size);
  
    draw_line(240, 240, xcoord as i16, ycoord as i16);
  }