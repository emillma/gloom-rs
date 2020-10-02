#version 430 core

smooth in vec4 theColor;
smooth in vec3 theNormal;
smooth in vec3 theLightSource;
smooth in vec3 V;

vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));
vec4 tmp;
float kd = 0.5;
float ks = 0.1;
out vec4 outColor;
void main()
{   
    tmp = theColor * max(dot(theNormal, -normalize(theLightSource)), 0.);
    tmp[3] = 1.;
    outColor = tmp;
}