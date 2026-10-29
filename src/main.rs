#![allow(warnings)]
#[macro_use]

extern crate glium;
use std::ops::Index;
use std::process::Output;
use std::{f32::consts::E, path::Display};

use glium::Surface;
use glium::texture::Texture2d;
use glium::texture::RawImage2d;
use glium::texture::Texture2dArray;
use glium::backend::Facade;
use bmp_rust::bmp::BMP;
use image::GenericImageView;
use std::ptr;

// Function to print the type of an object, used during debugging 

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

// Vertex struct. Terrain refers to the type of terrain the vertices should draw, from the  

#[derive(Copy, Clone)]
struct Vertex {
        position: [f32; 2],
        tex_coords: [f32; 2],
        color: f32,
        terrain: f32,
        relposscreen: [f32;2],
}
implement_vertex!(Vertex, position, tex_coords, color, terrain, relposscreen);

// Function to generate a single-dimensional vector of verticies from a Victoria 2 terrain map file.
// In the terrian map file each type of terrain has a colour assigned to it. This function goes over each colour pixel in the map, from top
// left to bottom right until it finds a colour that hasn't been mapped yet  
// Once an unmapped pixel is detected, it goes from that pixel to the right until it hits a pixel of another colour, at which point it starts going
// Down. When it hits a pixel of another colour going down, it marks that entire rectangle as having been mapped, and creates 6 vertices 
// Which represent the area. This repeats until it reaches the last pixel  


fn get_map() -> Vec<Vertex> {
	let mut returnmap = vec![];


	let mut bmp_from_file = BMP::new_from_file("map/terrain.bmp").unwrap();
	
	let dib_header = bmp_from_file.get_dib_header().unwrap();
	let width = dib_header.width;
	let height = dib_header.height;
	
    // NOTE: Each pixel has a u8 colour, which is also the index of the texture the pixel is supposed to be 
 
    let colorarray = bmp_from_file.get_pixel_data().unwrap();
    
    // The colour of the pixel the main loop is in

    let mut selectedcolor = colorarray[0][0].clone();

    // List of every unique colour over 64 encountered, for debugging 

    let mut colorstrackingarray: Vec<Vec<u8>> = Vec::new();

    let colorwhite = colorarray[0][0].clone();
    println!("{}", colorwhite[0]);
    //let mut c = 0.0;

    // 2d vector of booleans, if the selected pixel is part of a vertex it's true, otherwise false

	let mut mapped = vec![vec![false; height as usize]; width as usize];

    // Width and height of selected rectangle 

	let mut widthofselected = 0;
	let mut heightofselected = 0;
    let mut linehasother = false;
	
    let mut rectanglecount = 0;

    // Loops through each pixel once

	for h in 0..(height) {
		for w in 0..(width)
         {

            // If the pixel has already been mapped, skip it 

			if (mapped[w as usize][h as usize] == false)
			{

                // Reset the selected rectangle variables 

                widthofselected = 0;
                heightofselected = 0;
				selectedcolor = colorarray[h as usize][w as usize].clone();
                
				mapped[w as usize][h as usize] = true;
				
				for w2 in 0..(width) 
				{
                    if(w+w2 < width)
                    { 
                        if (selectedcolor == colorarray[h as usize][(w + w2) as usize]){
                            widthofselected = widthofselected + 1;

                            // Set the pixel it's on as having been mapped 

                            mapped[(w + w2) as usize][h as usize] = true;
                        }
                        else {

                            // If the colour is different, break out of the loop  
                            break;
                        }
                    }
					else 
					{

                        // If the width hits the edge of the map, break out of the loop

						break;
					}
				}

                // The rectangle selector for going down. Mostly the same as the previous section, except when it moves down it 
                // has to check that the whole line is the same colour   

                for h2 in 0..height 
                {
                    if (h + h2 >= height) { break };
                    linehasother = false;
                    for w3 in 0..widthofselected
                    {
                        if (selectedcolor != colorarray[(h + h2)as usize][(w + w3) as usize])
                            {
                                linehasother = true;
                                break;
                            }
                    }

                    heightofselected = heightofselected + 1;

                    if (linehasother == true)
                    {
                    break;
                    }

                    for w4 in 0..widthofselected {
                        mapped[(w + w4) as usize][(h + h2)  as usize] = true;
                    }
                    
                }


                let mut terrainnumber: f32 = 0.0;

                // Check if the selected colour is already in the colour array, and that 
                // it has a corresponding terrain texture (it's value is not 65 or above)
                // If it isn't, add it to the list of colours 

                if (selectedcolor[0] >= 64)
                {
                    
                    if !colorstrackingarray.contains(&selectedcolor)
                    {
                            colorstrackingarray.push(selectedcolor.clone());
                    }
                    //terrainnumber = 65.0;
                }
                else{
                    terrainnumber = selectedcolor[0] as f32;
                }

                if (selectedcolor[0] <= 64 ){

                // Generate the information needed for the vertex structs for the rectangle that was selected 

                let relpos_w = w as f32 / width as f32;
                let relpos_h = h as f32 / height as f32;
                let relpos_w2 = (w as f32 + widthofselected as f32) / width as f32;
                let relpos_h2 = (h as f32 + heightofselected as f32) / height as f32;

				let mut norm_w = w as f32 / width as f32 * 2.0 - 1.0;
                let mut norm_h = h as f32 / height as f32 * 2.0 - 1.0;
                let mut norm_w2 = (w + widthofselected) as f32 / width as f32 * 2.0 - 1.0;
                let mut norm_h2 = (h + heightofselected) as f32 / height as f32 * 2.0 - 1.0;
                
                let zoom = 0.95;
                let left = 0.0 / width as f32;
                let up = 0.0 / height as f32;

                norm_w = (norm_w + left) * zoom;
                norm_h = (norm_h - up) * zoom;
                norm_w2 = (norm_w2 + left) * zoom;
                norm_h2 = (norm_h2 - up) * zoom;
                
                let div: f32 = 25.6 * 0.55;
                
                let mut norm_w_t = w as f32 / div;// / (256.0 * width as f32);
                let mut norm_h_t = h as f32 / div;// / 256.0;
                let mut norm_w2_t = (w + widthofselected) as f32 / div;// / 256.0;
                let mut norm_h2_t = (h + heightofselected) as f32 / div;// / 256.0;


                // Add the six verticies to the final vector of verticies
                
                returnmap.push(Vertex { position: [norm_w, norm_h], tex_coords: [norm_w_t, norm_h_t], color: selectedcolor[0] as f32, terrain: terrainnumber, relposscreen: [relpos_w,relpos_h] });
                returnmap.push(Vertex { position: [norm_w2, norm_h], tex_coords: [norm_w2_t, norm_h_t], color: selectedcolor[0] as f32, terrain: terrainnumber,  relposscreen: [relpos_w2,relpos_h]});
                returnmap.push(Vertex { position: [norm_w2, norm_h2], tex_coords: [norm_w2_t, norm_h2_t], color: selectedcolor[0] as f32, terrain: terrainnumber,  relposscreen: [relpos_w2,relpos_h2]});

                returnmap.push(Vertex { position: [norm_w2, norm_h2], tex_coords: [norm_w2_t, norm_h2_t], color: selectedcolor[0] as f32, terrain: terrainnumber,  relposscreen: [relpos_w2,relpos_h2]});
                returnmap.push(Vertex { position: [norm_w, norm_h2], tex_coords: [norm_w_t, norm_h2_t], color: selectedcolor[0] as f32, terrain: terrainnumber,  relposscreen: [relpos_w,relpos_h2]});
                returnmap.push(Vertex { position: [norm_w, norm_h], tex_coords: [norm_w_t, norm_h_t], color: selectedcolor[0] as f32, terrain: terrainnumber,  relposscreen: [relpos_w,relpos_h]});
                                
                //}

                rectanglecount = rectanglecount + 1;
                }
                }
            }
		}
        //println!("{}", h);
    
    println!("rectanglecount: {}", rectanglecount);
    println!("Height: {}, Width: {}", height, width);

    colorstrackingarray.sort();
    /*for p in colorstrackingarray
    {
        for f in p
        {
            print!("colour stacking array {}",f);
        }
        println!("{}","");
    }*/

    // Check for if any pixels in the map weren't covered by the algorithm 

    for w in mapped {
        for h in w
        {
        if h == false {
            println!("{}", "Fail");
            break;
        }
        }
    }

	return returnmap;
}

fn get_textures<F: ?Sized>(input_display: &F) -> Texture2dArray  // 5 horizontal 8 vertical
            where F: Facade
    {
    let mut outputvec: Vec<RawImage2d<'_, u8>> = Vec::new();


    let texturesheet = image::load(std::io::Cursor::new(&include_bytes!("textures/texturesheet.tga")),image::ImageFormat::Tga).unwrap();

    let mut index = 0;
    for h in 1..9   
        {
        for w in 1..9
            {
            let mut output1 = image::SubImage::new(&texturesheet, 256 *(w - 1),256 *(h - 1), 256, 256);
            let output2 = output1.to_image();
            let output_dimensions = output2.dimensions();
            //let output2 = output1.to_image().into_raw();

            let image = glium::texture::RawImage2d::from_raw_rgba(output2.into_raw(), output_dimensions);
            
            //let texture = glium::Texture2d::new(input_display, image).unwrap();


            //print_type_of(image);
            outputvec.push(image);
            //outputvec[index] = texture;
            index = index + 1;
            //print_type_of(texture);
        }
    }
    let texturesarray = Texture2dArray::new(input_display, outputvec).unwrap();
    return texturesarray;

}

fn main() {

    let event_loop = glium::winit::event_loop::EventLoop::builder().build().expect("event loop building");
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().with_title("Victoria 2 map renderer").build(&event_loop);

    //print_type_of(display);

    let texturesarray = get_textures(&display);

    let imagecolormap = image::load(std::io::Cursor::new(&include_bytes!("textures/colormap.png")), image::ImageFormat::Png).unwrap().to_rgba8();
    let imagecolormapdimensions = imagecolormap.dimensions();

    // The terrain map is mixed with a custom colour filter map. 

    let imagecolormap = glium::texture::RawImage2d::from_raw_rgba(imagecolormap.into_raw(), imagecolormapdimensions);

    let colormaptexture = glium::Texture2d::new(&display,imagecolormap).unwrap();
        
    println!("{}", "Starting");
	let map: Vec<Vertex> = get_map();
    println!("{}", "Done");

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
    let vertex_buffer = glium::VertexBuffer::new(&display, &map).unwrap();

    let vertex_shader = r#"
    #version 140

    // Mostly just passes in the information from the vertex struct 

    in vec2 position;
    in vec2 tex_coords;
    in vec2 relposscreen;
    in float terrain;
    in float color;

    //out float v_color;
    out vec2 v_tex_coords;
    out float v_terrain;
    out vec2 v_relposscreen;

    void main() { 
        v_relposscreen = relposscreen;
        v_tex_coords = tex_coords;
        gl_Position = vec4(position, -1.0, 1.0);
        //v_color = color;
        v_terrain = terrain;
    }
    "#;

    let fragment_shader = r#"
    #version 140
    
    // Gets texture coords, terrian and screen position from previous shader. Textures and colourmap get passed in

    in vec2 v_tex_coords;
    in float v_terrain;
    in vec2 v_relposscreen;
    out vec4 colour;

    uniform sampler2DArray textures; 
    uniform sampler2D colormaptexture;

    // Sets the colour of the pixel to the colour of the coordinates on the terrain texture mixed with the colour map 

    void main() {
        int v_terrain_int = int(v_terrain);
        if (v_terrain_int != 65) {
            vec4 tex1 = texture(textures, vec3(v_tex_coords, v_terrain_int));
            vec4 tex2 = texture(colormaptexture, v_relposscreen);
            colour = mix(tex1, tex2, 0.45);
        }
        else 
        {
        colour = vec4(5.0,5.0,15,0.45);
        }
    }
    "#;

    let program = glium::Program::from_source (&display, vertex_shader, fragment_shader, None).unwrap();
    
    let uniforms = uniform! {};

    //=====================

    #[allow(deprecated)]

    // Rendering loop, mostly untouched 

    event_loop.run(move |ev, window_target| {
        match ev {
            glium::winit::event::Event::WindowEvent { event, .. } => match event {
                glium::winit::event::WindowEvent::CloseRequested => {
                    window_target.exit();
                },

                glium::winit::event::WindowEvent::RedrawRequested => {

                    let mut target = display.draw();
                    target.clear_color(0.0, 0.0, 0.2, 1.0);
					
                    let uniforms = uniform! {
                        matrix: [
                            [1.0 , 0.0 , 0.0 , 0.0],
                            [0.0, 1.0, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [0.5 , 0.0, 0.0, 1.0f32],
                        ],
                        textures: &texturesarray,
                        colormap: &colormaptexture,
                    };
					

                    target.draw(&vertex_buffer, &indices, &program, &uniforms,
                                &Default::default()).unwrap();
                    target.finish().unwrap();
                },
                
                glium::winit::event::WindowEvent::Resized(window_size) => {
                    display.resize(window_size.into());
                },
                _ => (),
            },
            
            glium::winit::event::Event::AboutToWait => {
                window.request_redraw();
            },
            _ => (),
        }
    })
    .unwrap();
    
    
}