#version 460 core

smooth in vec4 theColor;
out vec4 outColor;
void main()
{
    outColor = vec4(theColor.x, theColor.y, theColor.z, 0.7);;
}