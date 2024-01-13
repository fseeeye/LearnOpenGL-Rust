#version 330 core

out vec4 frag_color;

in VS_OUT {
    vec3 world_pos;
    vec3 normal;
    vec2 texture_coord;
} fs_in;

uniform vec3 camera_pos;
// PBR Material parameters
uniform bool enable_pbr_map;
uniform vec3 albedo;
uniform float metallic;
uniform float roughness;
uniform float ao;
uniform sampler2D albedo_map;
uniform sampler2D normal_map;
uniform sampler2D metallic_map;
uniform sampler2D roughness_map;
uniform sampler2D ao_map;
// Lights parameters
const int LIGHTS_NUM = 4;
uniform vec3 light_positions[LIGHTS_NUM];
uniform vec3 light_colors[LIGHTS_NUM];

const float PI = 3.14159265359;
const float gamma = 2.2;

// D term use GGX Model
float DistributionTerm(vec3 N, vec3 H, float roughness)
{
    float a2     = roughness*roughness;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;

    float nom    = a2;
    float denom  = (NdotH2 * (a2 - 1.0) + 1.0);
    denom        = PI * denom * denom;
    return nom / denom;
}

// F term use Schlick’s approximation
vec3 FresnelTerm(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;
    return nom / denom;
}

// Geometry term use The Smith shadowing-masking algorithm
float GeometryTerm(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);

    float ggx_view = GeometrySchlickGGX(NdotV, roughness);
    float ggx_light = GeometrySchlickGGX(NdotL, roughness);
    return ggx_view * ggx_light;
}

// Get tangent-normals to world-space.
vec3 GetNormalFromMap()
{
    vec3 tangent_normal = texture(normal_map, fs_in.texture_coord).xyz;
    tangent_normal = tangent_normal * 2.0 - 1.0; // from [0, 1] to [-1, 1]

    vec3 Edge1  = dFdx(fs_in.world_pos);
    vec3 Edge2  = dFdy(fs_in.world_pos);
    vec2 delta_uv1 = dFdx(fs_in.texture_coord);
    vec2 delta_uv2 = dFdy(fs_in.texture_coord);

    vec3 N   = normalize(fs_in.normal);
    vec3 T   = normalize(Edge1*delta_uv2.t - Edge2*delta_uv1.t);
    vec3 B   = -normalize(cross(N, T));
    mat3 TBN = mat3(T, B, N);

    return normalize(TBN * tangent_normal);
}

void main()
{
    vec3 V = normalize(camera_pos - fs_in.world_pos);
    vec3 N = normalize(fs_in.normal);
    vec3 albedo = albedo;
    float metallic = metallic;
    float roughness = roughness;
    float ao = ao;
    if (enable_pbr_map) {
        N = GetNormalFromMap();
        albedo = pow(texture(albedo_map, fs_in.texture_coord).rgb, vec3(2.2)); // convert albedo from sRGB to linear
        metallic = texture(metallic_map, fs_in.texture_coord).r;
        roughness = texture(roughness_map, fs_in.texture_coord).r;
        ao = texture(ao_map, fs_in.texture_coord).r;
    }

    // Calculate "Reflectance" at normal incidence;
    // If dia-electric use F0 of 0.04 and if it's a metal, use the albedo color as F0 (metallic workflow)
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);

    vec3 Lo = vec3(0.0);
    for(int i = 0; i < LIGHTS_NUM; ++i) // traverse all lights
    {
        vec3 L = normalize(light_positions[i] - fs_in.world_pos);
        float cosTheta = max(dot(N, L), 0.0);
        vec3 H = normalize(V + L);

        float distance = length(light_positions[i] - fs_in.world_pos);
        float attenuation = 1.0 / (distance * distance);
        vec3 Li = light_colors[i] * attenuation;

        /* Calc specular part */
        float D = DistributionTerm(N, H, roughness);
        vec3  F = FresnelTerm(clamp(dot(H, V), 0.0, 1.0), F0);
        float G = GeometryTerm(N, V, L, roughness);
        vec3 numerator    = D * G * F; 
        float denominator = 4.0 * max(dot(N, V), 0.0) * cosTheta + 0.0001; // add 0.0001 to prevent divide by zero
        vec3 specular = numerator / denominator;

        /* Calc diffuse part */
        // kS is equal to Fresnel
        vec3 Ks = F;
        // for energy conservation, the diffuse and specular light can't
        // be above 1.0 (unless the surface emits light); to preserve this
        // relationship the diffuse component (kD) should equal 1.0 - kS.
        vec3 Kd = vec3(1.0) - Ks;
        // multiply kD by the inverse metalness such that only non-metals 
        // have diffuse lighting, or a linear blend if partly metal (pure metals
        // have no diffuse light).
        Kd *= 1.0 - metallic;

        vec3 diffuse = (Kd * albedo) / PI;

        /* The Rendering Equation */
        vec3 BRDF = diffuse + specular;
        Lo += Li * BRDF * cosTheta;
    }

    // Add ambient lighting
    Lo += vec3(0.03) * albedo * ao;

    // Reinhard Tone Mapping
    Lo = Lo / (Lo + vec3(1.0));
    // Gamma correct
    Lo = pow(Lo, vec3(1.0 / gamma));

    frag_color = vec4(Lo, 1.0);
}
