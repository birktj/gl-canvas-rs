#[macro_use]
extern crate glium;
extern crate lyon;
extern crate nalgebra as na;

use glium::Surface;

use std::rc::Rc;

use lyon::math::point;
use lyon::path::builder::*;
use lyon::path::default::Path;
use lyon::tessellation::*;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }
}

pub struct RenderContext {
    display: glium::Display,
    target: glium::Frame,
    program: glium::Program,
    transform_matrix: na::Similarity2<f32>,
    matrix_stack: Vec<na::Similarity2<f32>>,
    stroke_col: Color,
    fill_col: Color,
    path: lyon::path::default::Builder,
}

impl Drop for RenderContext {
    fn drop(&mut self) {
        self.target.set_finish().unwrap();
    }
}

impl RenderContext {
    pub fn new(display: glium::Display) -> RenderContext {
        let frame = display.draw();
        let program =
            glium::Program::from_source(&display, VERTEX_SHADER, FRAGMENT_SHADER, None).unwrap();
        RenderContext {
            display: display,
            target: frame,
            program: program,
            transform_matrix: na::Similarity2::identity(),
            matrix_stack: Vec::new(),
            stroke_col: Color::new(0.0, 0.0, 0.0, 1.0),
            fill_col: Color::new(0.0, 0.0, 0.0, 1.0),
            path: Path::builder(),
        }
    }

    pub fn render(&mut self) {
        self.target.set_finish().unwrap();
        self.target = self.display.draw();
    }

    pub fn clear<C: Into<Color>>(&mut self, color: C) {
        let color: Color = color.into();
        self.target.clear_color(color.r, color.g, color.b, color.a);
    }

    pub fn push_matrix(&mut self) {
        self.matrix_stack.push(self.transform_matrix);
    }
    pub fn pop_matrix(&mut self) {
        self.transform_matrix = self.matrix_stack
            .pop()
            .unwrap_or(na::Similarity2::identity());
    }

    pub fn rotate(&mut self, ang: f32) {
        self.transform_matrix
            .append_rotation_mut(&na::UnitComplex::from_angle(ang));
    }
    pub fn scale(&mut self, factor: f32) {
        self.transform_matrix.append_scaling_mut(factor);
    }
    pub fn translate(&mut self, x: f32, y: f32) {
        self.transform_matrix
            .append_translation_mut(&na::Translation2::from_vector(na::Vector2::new(x, y)));
    }
    pub fn transform(&mut self, matrix: na::Similarity2<f32>) {
        self.transform_matrix *= matrix;
    }
    pub fn reset_transform(&mut self) {
        self.transform_matrix = na::Similarity2::identity();
    }

    fn render_matrix(&self) -> [[f32;3];3] {
        let (w, h) = self.display.get_framebuffer_dimensions();
        let mut matrix = na::Transform2::identity().unwrap();
        matrix.append_translation_mut(&na::Vector2::new(-(w as f32/2.0), h as f32/2.0));
        matrix *= na::Matrix3::from([[1.0,0.0,0.0],
                                 [0.0,-1.0,0.0],
                                 [0.0,0.0,1.0]]);
        matrix.append_nonuniform_scaling_mut(&na::Vector2::new(2.0/w as f32, 2.0/h as f32));

        matrix.into()
    }

    pub fn dimensions(&self) -> (f32, f32) {
        let (w, h) = self.display.get_framebuffer_dimensions();
        (w as f32, h as f32)
    }

    pub fn stroke_color<C: Into<Color>>(&mut self, color: C) {
        self.stroke_col = color.into();
    }
    pub fn fill_color<C: Into<Color>>(&mut self, color: C) {
        self.fill_col = color.into();
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to(point(x, y));
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to(point(x, y));
    }

    pub fn stroke(&mut self) {
        let path = self.path.build_and_reset();

        let mut tessellator = StrokeTessellator::new();

        let mut mesh: VertexBuffers<Vertex, u16> = VertexBuffers::new();
        tessellator.tessellate_path(
            path.path_iter(),
            &StrokeOptions::tolerance(0.1).with_line_width(2.0),
            &mut BuffersBuilder::new(&mut mesh, |vertex: StrokeVertex| Vertex {
                position: vertex.position.to_array(),
            }),
        );

        let verticies = glium::VertexBuffer::new(&self.display, &mesh.vertices).unwrap();
        let indices = glium::IndexBuffer::new(
            &self.display,
            glium::index::PrimitiveType::TrianglesList,
            &mesh.indices,
        ).unwrap();


        let uniforms = uniform!{
            color: [self.stroke_col.r, self.stroke_col.g, self.stroke_col.b, self.stroke_col.a],
            matrix: self.render_matrix()
        };

        self.target
            .draw(
                &verticies,
                &indices,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
    }
    pub fn fill(&mut self) {
        let path = self.path.build_and_reset();

        let mut tessellator = FillTessellator::new();

        let mut mesh: VertexBuffers<Vertex, u16> = VertexBuffers::new();
        tessellator.tessellate_path(
            path.path_iter(),
            &FillOptions::tolerance(0.1),
            &mut BuffersBuilder::new(&mut mesh, |vertex: FillVertex| Vertex {
                position: vertex.position.to_array(),
            }),
        );

        let verticies = glium::VertexBuffer::new(&self.display, &mesh.vertices).unwrap();
        let indices = glium::IndexBuffer::new(
            &self.display,
            glium::index::PrimitiveType::TrianglesList,
            &mesh.indices,
        ).unwrap();


        let uniforms = uniform!{
            color: [self.fill_col.r, self.fill_col.g, self.fill_col.b, self.fill_col.a],
            matrix: self.render_matrix()
        };

        self.target
            .draw(
                &verticies,
                &indices,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
    }
}

const VERTEX_SHADER: &'static str = r#"
    #version 140

    in vec2 position;

    uniform mat3 matrix;

    void main() {
        gl_Position = vec4(matrix * vec3(position, 1.0), 1.0);
    }
"#;

const FRAGMENT_SHADER: &'static str = r#"
    #version 140
    
    out vec4 f_color;

    uniform vec4 color;

    void main() {
        //color = vec4(1.0, 0.0, 0.0, 1.0);
        f_color = color;
    }
"#;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
