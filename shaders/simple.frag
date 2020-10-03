#version 430 core

smooth in vec4 theColor;
smooth in vec3 N;
smooth in vec3 L;
smooth in vec3 V;
smooth in vec3 R;

vec4 tmp;

float kd = 0.8;
float diffuse_gain;

float specular_gain;
float ks = 0.3;
float alpha = 12.;

float ambient = 0.1;
out vec4 outColor;
void main()
{   
    diffuse_gain = max(dot(N, L), 0.);
    specular_gain = pow(max(dot(R, V), 0.), alpha);

    tmp = theColor * min(ambient + kd * diffuse_gain + ks * specular_gain, 1.);
    tmp[3] = theColor[3];
    outColor = tmp;
}