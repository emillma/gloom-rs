#version 430 core

smooth in vec4 theColor;
smooth in vec3 theNormal;

vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));
vec4 tmp;

out vec4 outColor;
void main()
{
    tmp = theColor * max(dot(theNormal, -lightDirection), 0.);
    tmp[3] = 1.;
    outColor = tmp;
}