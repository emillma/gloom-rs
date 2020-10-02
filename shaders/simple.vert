#version 430 core

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec4 vertex_color;
layout(location = 2) in vec3 vertex_normal;


uniform mat4 ViewProjection;
uniform vec4 LightSource;

out vec4 theColor;
out vec3 N;
out vec3 L;
out vec3 V;
out vec3 R;

mat4 matrix = mat4(
   1.0, 0.0, 0.0, 0.0, // first column (not row!)
   0.0, 1.0, 0.0, 0.0,
   0.0, 0.0, 1.0, 0.0,
   0.0, 0.0, 0.0, 1.0
);

void main()
{
    gl_Position = ViewProjection *  vec4(vertex_position, 1.);

    theColor = vertex_color;
    // theNormal = vec3(ViewProjection *  vec4(vertex_normal, 0.));
    N = -vec3(ViewProjection *  vec4(vertex_normal, 0.));
    L =  normalize(vec3(LightSource - gl_Position));
    V = normalize(vec3(gl_Position));
    R = 2 * dot(L, N) * N - L;
}
