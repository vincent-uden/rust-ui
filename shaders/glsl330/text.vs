#version 330 core 
layout (location = 0) in vec4 vertex; // <vec2 pos, vec2 tex>
layout (location = 1) in vec2 instance_position; // per-instance position
layout (location = 2) in vec2 instance_size; // per-instance size
layout (location = 3) in vec2 instance_atlas_coords; // per-instance atlas UV coords
layout (location = 4) in vec2 instance_atlas_size; // per-instance atlas UV size

out vec2 TexCoords;

uniform mat4 projection;

void main()
{
    // Transform unit quad to character position and size
    vec2 world_pos = instance_position + vertex.xy * instance_size;
    gl_Position = projection * vec4(world_pos, 0.0, 1.0);
    
    // Map unit quad UV to character's atlas UV region
    TexCoords = instance_atlas_coords + vertex.zw * instance_atlas_size;
}
