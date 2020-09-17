#version 460 core

smooth in vec4 theColor;
out vec4 outColor;
void main()
{
    outColor = theColor;
}