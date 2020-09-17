#version 460 core

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec4 vertex_color;


uniform mat4 ViewProjection;

out vec4 theColor;

mat4 matrix = mat4(
   1.0, 0.0, 0.0, 0.0, // first column (not row!)
   0.0, 1.0, 0.0, 0.0,
   0.0, 0.0, 1.0, 0.0,
   0.0, 0.0, -5.0, 1.0
);

void main()
{
    gl_Position = ViewProjection * matrix *  vec4(vertex_position, 1.);
    theColor = vertex_color;
}
