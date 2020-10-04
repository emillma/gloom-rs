#version 430 core

layout(location = 0) in vec3 VertexPosition;
layout(location = 1) in vec4 vertex_color;
layout(location = 2) in vec3 vertex_normal;


// uniform mat4 CameraTranslation;
uniform mat4 ViewProjectionMatrix;
uniform mat4 SceneTransfrom;
uniform vec3 CameraPosition;
uniform vec3 LightSource;

// mat4 ViewProjection = CameraIntrisinc * CameraTranslation;

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
    gl_Position = ViewProjectionMatrix *  SceneTransfrom * vec4(VertexPosition, 1.);

    theColor = vertex_color;
    // theNormal = vec3(ViewProjection *  vec4(vertex_normal, 0.));
    N = normalize(vec3(SceneTransfrom * vec4(vertex_normal, 0.)));
    L =  normalize(LightSource - VertexPosition);
    V = normalize(CameraPosition - VertexPosition);
    R = 2 * dot(L, N) * N - L;
}
