#version 430 core

layout (location = 0) in vec3 aPos;

void main()
{
    gl_Position = vec4(0.5-aPos.x, -aPos.y, aPos.z, 1.0);
}