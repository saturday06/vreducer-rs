use byteorder::{ReadBytesExt, LE};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;

pub const CHUNK_TYPE: u32 = 0x4e4942;

pub fn for_each_material_index_references<F>(gltf: &mut Value, mut f: F)
where
    F: FnMut(&mut serde_json::Number),
{
    for mesh in gltf
        .get_mut("meshes")
        .and_then(|v| v.as_array_mut())
        .unwrap_or(&mut Vec::new())
    {
        for primitive in mesh
            .get_mut("primitives")
            .and_then(|v| v.as_array_mut())
            .unwrap_or(&mut Vec::new())
        {
            if let Some(Value::Number(ref mut index)) = primitive.get_mut("material") {
                f(index);
            }
        }
    }
}

pub fn for_each_accessor_index_references<F>(gltf: &mut Value, mut f: F)
where
    F: FnMut(&mut serde_json::Number),
{
    for skin in gltf
        .get_mut("skins")
        .and_then(|v| v.as_array_mut())
        .unwrap_or(&mut Vec::new())
    {
        if let Some(Value::Number(ref mut index)) = skin.get_mut("inverseBindMatrices") {
            f(index);
        }
    }

    for mesh in gltf
        .get_mut("meshes")
        .and_then(|v| v.as_array_mut())
        .unwrap_or(&mut Vec::new())
    {
        for primitive in mesh
            .get_mut("primitives")
            .and_then(|v| v.as_array_mut())
            .unwrap_or(&mut Vec::new())
        {
            if let Some(Value::Number(ref mut index)) = primitive.get_mut("indices") {
                f(index);
            }

            for (_, value) in primitive
                .get_mut("attributes")
                .and_then(|v| v.as_object_mut())
                .unwrap_or(&mut serde_json::map::Map::new())
            {
                if let Value::Number(ref mut index) = value {
                    f(index);
                }
            }

            for target in primitive
                .get_mut("targets")
                .and_then(|v| v.as_array_mut())
                .unwrap_or(&mut Vec::new())
            {
                for (_, value) in target
                    .as_object_mut()
                    .unwrap_or(&mut serde_json::map::Map::new())
                {
                    if let Value::Number(ref mut index) = value {
                        f(index);
                    }
                }
            }
        }
    }
}

pub fn for_each_sampler_index_references<F>(gltf: &mut Value, mut f: F)
where
    F: FnMut(&mut serde_json::Number),
{
    for texture in gltf
        .get_mut("textures")
        .and_then(|v| v.as_array_mut())
        .unwrap_or(&mut Vec::new())
    {
        if let Some(Value::Number(ref mut index)) = texture.get_mut("sampler") {
            f(index);
        }
    }
}

pub fn for_each_image_index_references<F>(gltf: &mut Value, mut f: F)
where
    F: FnMut(&mut serde_json::Number),
{
    for texture in gltf
        .get_mut("textures")
        .and_then(|t| t.as_array_mut())
        .unwrap_or(&mut Vec::new())
    {
        if let Some(Value::Number(ref mut index)) = texture.get_mut("source") {
            f(index);
        }
    }
}

pub fn for_each_texture_index_references<F>(gltf: &mut Value, mut f: F)
where
    F: FnMut(&mut serde_json::Number),
{
    for material in gltf
        .get_mut("materials")
        .and_then(|m| m.as_array_mut())
        .unwrap_or(&mut Vec::new())
    {
        for key in &["baseColorTexture", "metallicRoughnessTexture"] {
            if let Some(Value::Number(ref mut index)) = material
                .get_mut("pbrMetallicRoughness")
                .and_then(|v| v.get_mut(*key))
                .and_then(|v| v.get_mut("index"))
            {
                f(index)
            }
        }

        for key in &["normalTexture", "occlusionTexture", "emissiveTexture"] {
            if let Some(Value::Number(ref mut index)) =
                material.get_mut(*key).and_then(|v| v.get_mut("index"))
            {
                f(index)
            }
        }
    }

    if let Some(Value::Number(ref mut index)) = gltf
        .get_mut("extensions")
        .and_then(|v| v.get_mut("VRM"))
        .and_then(|v| v.get_mut("meta"))
        .and_then(|v| v.get_mut("texture"))
    {
        f(index)
    }

    for material_properties in gltf
        .get_mut("extensions")
        .and_then(|v| v.get_mut("VRM"))
        .and_then(|v| v.get_mut("materialProperties"))
        .and_then(|v| v.as_array_mut())
        .unwrap_or(&mut Vec::new())
    {
        // https://github.com/Santarh/MToon/blob/bf08610b38e27ae915b0613ee4dfb574bc8e381b/MToon/Resources/Shaders/MToon.shader
        for key in &[
            "_MainTex",
            "_ShadeTexture",
            //"_BumpMap",
            "_ReceiveShadowTexture",
            "_ShadingGradeTexture",
            //"_SphereAdd",
            "_EmissionMap",
            "_OutlineWidthTexture",
        ] {
            if let Some(Value::Number(ref mut index)) = material_properties
                .get_mut("textureProperties")
                .and_then(|v| v.get_mut(*key))
            {
                f(index)
            }
        }
    }
}

pub fn for_each_buffer_view_index_references<F>(gltf: &mut Value, mut f: F)
where
    F: FnMut(&mut serde_json::Number),
{
    for accessor in gltf
        .get_mut("accessors")
        .and_then(|v| v.as_array_mut())
        .unwrap_or(&mut Vec::new())
    {
        if let Some(Value::Number(ref mut index)) = accessor.get_mut("bufferView") {
            f(index);
        }

        if let Some(Value::Number(ref mut index)) = accessor
            .get_mut("sparse")
            .and_then(|v| v.get_mut("indices"))
            .and_then(|v| v.get_mut("bufferView"))
        {
            f(index);
        }

        if let Some(Value::Number(ref mut index)) = accessor
            .get_mut("values")
            .and_then(|v| v.get_mut("indices"))
            .and_then(|v| v.get_mut("bufferView"))
        {
            f(index);
        }
    }

    for image in gltf
        .get_mut("images")
        .and_then(|v| v.as_array_mut())
        .unwrap_or(&mut Vec::new())
    {
        if let Some(Value::Number(ref mut index)) = image.get_mut("bufferView") {
            f(index);
        }
    }
}

pub fn for_each_buffer_index_references<F>(gltf: &mut Value, mut f: F)
where
    F: FnMut(&mut serde_json::Number),
{
    for buffer_view in gltf
        .get_mut("bufferViews")
        .and_then(|v| v.as_array_mut())
        .unwrap_or(&mut Vec::new())
    {
        if let Some(Value::Number(ref mut index)) = buffer_view.get_mut("buffer") {
            f(index);
        }
    }
}

macro_rules! clean_resources {
    ($generator_function: ident, $resource_pointer: expr, $json: expr) => {{
        let mut json = $json.clone();
        let mut original_indexes = BTreeSet::new();
        $generator_function(&mut json, |index| {
            if let Some(i) = index.as_u64() {
                original_indexes.insert(i);
            } else {
                println!("Too large {} index: {:?}", $resource_pointer, index);
            }
        });

        let len = json
            .pointer($resource_pointer)
            .and_then(|v| v.as_array())
            .map(|t| t.len())
            .unwrap_or(0) as u64;
        for index in (0..len).rev() {
            if !original_indexes.contains(&index) {
                json.pointer_mut($resource_pointer)
                    .and_then(|v| v.as_array_mut())
                    .map(|t| t.remove(index as usize));
            }
        }

        let mut index_map = BTreeMap::new();
        for (reduced_index, original_index) in original_indexes.iter().enumerate() {
            index_map.insert(original_index, reduced_index);
        }

        $generator_function(&mut json, |index| {
            if let Some(i) = index.as_u64() {
                *index = index_map[&i].into();
            } else {
                println!("Too large {} index: {:?}", $resource_pointer, index);
            }
        });

        (
            json,
            original_indexes
                .iter()
                .map(|index| *index)
                .collect::<Vec<_>>(),
        )
    }};
}

pub fn fix_extension_vrm(gltf_: Value) -> Value {
    let mut gltf = gltf_.clone();

    if let Some(extensions_used) = gltf
        .get_mut("extensionsUsed")
        .and_then(|v| v.as_array_mut())
    {
        if !extensions_used.contains(&Value::String("VRM".into())) {
            extensions_used.push(Value::String("VRM".into()));
        }
    } else {
        gltf["extensionsUsed"] = vec!["VRM"].into();
    }

    if !gltf
        .pointer("extensions/VRM/meta")
        .map(|v| v.is_object())
        .unwrap_or(false)
    {
        gltf["extensions"]["VRM"]["meta"] = serde_json::map::Map::new().into();
    };

    for (key, value) in &[
        ("title", ""),
        ("version", ""),
        ("author", ""),
        ("contactInformation", ""),
        ("reference", ""),
        ("allowedUserName", "OnlyAuthor"),
        ("violentUssageName", "Disallow"),
        ("sexualUssageName", "Disallow"),
        ("commercialUssageName", "Disallow"),
        ("otherPermissionUrl", ""),
        ("licenseName", "Redistribution_Prohibited"),
        ("otherLicenseUrl", ""),
    ] {
        let meta = &mut gltf["extensions"]["VRM"]["meta"];
        if !meta.get_mut(*key).map(|v| v.is_string()).unwrap_or(false) {
            meta[*key] = (*value).into();
        }
    }

    gltf
}

pub fn clean(gltf: Value) -> Value {
    let (gltf, _) = clean_resources!(for_each_material_index_references, "/materials", gltf);
    let (gltf, _) = clean_resources!(for_each_texture_index_references, "/textures", gltf);
    let (gltf, _) = clean_resources!(for_each_image_index_references, "/images", gltf);
    let (gltf, _) = clean_resources!(for_each_accessor_index_references, "/accessors", gltf);
    let (gltf, _) = clean_resources!(for_each_sampler_index_references, "/samplers", gltf);
    let (gltf, _) = clean_resources!(for_each_buffer_view_index_references, "/bufferViews", gltf);
    fix_extension_vrm(gltf)
}

#[derive(Clone)]
pub struct BufferViewRegion {
    byte_offset: u64,
    byte_length: u64,
}

pub fn relocate_buffers(gltf_: Value) -> (Value, BufferRelocator) {
    let gltf = gltf_.clone();
    let (mut gltf, remaining_chunk_indexes) =
        clean_resources!(for_each_buffer_index_references, "/buffers", gltf);
    let mut buffer_view_regions_by_index = Vec::new();

    for buffer_view in gltf
        .get("bufferViews")
        .and_then(|v| v.as_array())
        .unwrap_or(&mut Vec::new())
    {
        if let (Some(buffer), byte_offset, Some(byte_length)) = (
            buffer_view.get("buffer").and_then(|v| v.as_u64()),
            buffer_view
                .get("byteOffset")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            buffer_view.get("byteLength").and_then(|v| v.as_u64()),
        ) {
            //println!("buffer_view: {} {} {}", buffer, byte_offset, byte_length);
            while buffer_view_regions_by_index.len() <= buffer as usize {
                buffer_view_regions_by_index.push(Vec::new());
            }

            buffer_view_regions_by_index
                .get_mut(buffer as usize)
                .map(|buffer_view_regions| {
                    buffer_view_regions.push(BufferViewRegion {
                        byte_offset,
                        byte_length,
                    })
                });
        }
    }

    buffer_view_regions_by_index = buffer_view_regions_by_index
        .iter()
        .map(|buffer_view_regions_| {
            let mut buffer_view_regions = buffer_view_regions_.clone();
            buffer_view_regions.sort_by(|l, r| {
                l.byte_offset
                    .cmp(&r.byte_offset)
                    .then(l.byte_length.cmp(&r.byte_length))
            });
            buffer_view_regions
        })
        .collect::<Vec<_>>();

    let mut deleted_buffer_view_regions_by_index = Vec::new();
    for buffer_view_regions in &buffer_view_regions_by_index {
        //println!("buffer_view_regions: len={}", buffer_view_regions.len());
        let mut deleted_buffer_view_regions = Vec::new();
        let mut next_offset = 0;
        for buffer_view_region in buffer_view_regions {
            // TODO: accessorからアラインメントを取得
            let alignment = 8;
            let aligned_next_offset = (next_offset + alignment - 1) / alignment * alignment;
            let aligned_byte_offset = buffer_view_region.byte_offset / alignment * alignment;
            if aligned_next_offset < aligned_byte_offset {
                //println!(
                //    "deleted buffer view region: start={}->{} end={}->{} len={}",
                //    next_offset,
                //    aligned_next_offset,
                //    buffer_view_region.byte_offset,
                //    aligned_byte_offset,
                //    aligned_byte_offset - aligned_next_offset,
                //);

                deleted_buffer_view_regions.push(BufferViewRegion {
                    byte_offset: aligned_next_offset,
                    byte_length: aligned_byte_offset - aligned_next_offset,
                });
            }
            if next_offset < buffer_view_region.byte_offset + buffer_view_region.byte_length {
                next_offset = buffer_view_region.byte_offset + buffer_view_region.byte_length;
            }
        }
        deleted_buffer_view_regions_by_index.push(deleted_buffer_view_regions);
    }

    for buffer_view in gltf
        .get_mut("bufferViews")
        .and_then(|v| v.as_array_mut())
        .unwrap_or(&mut Vec::new())
    {
        if let (Some(buffer), byte_offset) = (
            buffer_view.get_mut("buffer").and_then(|v| v.as_u64()),
            buffer_view
                .get_mut("byteOffset")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        ) {
            deleted_buffer_view_regions_by_index
                .get(buffer as usize)
                .map(|deleted_buffer_views| {
                    let shift = deleted_buffer_views
                        .iter()
                        .filter(|deleted_buffer_view| {
                            deleted_buffer_view.byte_offset + deleted_buffer_view.byte_length
                                <= byte_offset
                        })
                        .fold(0, |sum, deleted_buffer_view| {
                            sum + deleted_buffer_view.byte_length
                        });
                    buffer_view["byteOffset"] = (byte_offset - shift).into();
                });
        }
    }

    let mut remaining_buffer_view_regions_by_index = Vec::new();
    for (index, buffer_view_regions) in buffer_view_regions_by_index.iter().enumerate() {
        //println!("remaining_buffer_view_region: {}", index);
        let buffer_length = if let Some(buffer_view_region) = buffer_view_regions.last() {
            let alignment = 4;
            (buffer_view_region.byte_offset + buffer_view_region.byte_length + alignment - 1)
                / alignment
                * alignment
        } else {
            remaining_buffer_view_regions_by_index.push(Vec::new());
            continue;
        };

        let mut byte_offset = 0;
        let mut remaining_buffer_view_regions = Vec::new();
        for deleted_buffer_view_region in deleted_buffer_view_regions_by_index
            .get(index)
            .unwrap_or(&Vec::new())
        {
            if byte_offset < deleted_buffer_view_region.byte_offset {
                //println!(
                //    "add {} {}",
                //    byte_offset,
                //    deleted_buffer_view_region.byte_offset - byte_offset
                //);
                remaining_buffer_view_regions.push(BufferViewRegion {
                    byte_offset,
                    byte_length: deleted_buffer_view_region.byte_offset - byte_offset,
                });
            }
            byte_offset =
                deleted_buffer_view_region.byte_offset + deleted_buffer_view_region.byte_length
        }
        if buffer_length > byte_offset {
            remaining_buffer_view_regions.push(BufferViewRegion {
                byte_offset,
                byte_length: buffer_length - byte_offset,
            });
        }
        gltf["buffers"][index]["byteLength"] = remaining_buffer_view_regions
            .iter()
            .fold(0, |sum, r| sum + r.byte_length)
            .into();
        remaining_buffer_view_regions_by_index.push(remaining_buffer_view_regions);
    }

    let buffer_relocator = BufferRelocator::new(
        &gltf,
        remaining_chunk_indexes,
        remaining_buffer_view_regions_by_index,
    );
    (gltf, buffer_relocator)
}

pub struct BufferRelocator {
    remaining_chunk_indexes: Vec<u64>,
    remaining_buffer_view_regions_by_index: Vec<Vec<BufferViewRegion>>,
}

impl BufferRelocator {
    pub fn new(
        _gltf: &Value,
        remaining_chunk_indexes: Vec<u64>,
        remaining_buffer_view_regions_by_index: Vec<Vec<BufferViewRegion>>,
    ) -> BufferRelocator {
        BufferRelocator {
            remaining_chunk_indexes,
            remaining_buffer_view_regions_by_index,
        }
    }

    pub fn total_chunk_bytes(&self) -> u32 {
        self.remaining_buffer_view_regions_by_index
            .iter()
            .fold(0, |s, r| {
                s + 8 + r.iter().fold(0, |ss, rr| ss + rr.byte_length as u32)
            })
    }

    pub fn relocate<R>(
        &self,
        mut reader: R,
        total_bytes: u32,
    ) -> Result<Vec<Vec<u8>>, Box<std::error::Error>>
    where
        R: Read,
    {
        let mut chunk_index = 0;
        let mut offset = 0;
        let mut chunks = Vec::new();
        println!("relocate: {}/{}", offset, total_bytes);
        while offset < total_bytes {
            let chunk_length = reader.read_u32::<LE>()?;
            let chunk_type = reader.read_u32::<LE>()?;
            assert_eq!(chunk_type, CHUNK_TYPE);

            if self.remaining_chunk_indexes.contains(&chunk_index) {
                let mut chunk_bytes = Vec::new();
                let mut chunk_offset = 0;
                for remaining_buffer_view_region in self
                    .remaining_buffer_view_regions_by_index
                    .get(chunk_index as usize)
                    .unwrap_or(&Vec::new())
                {
                    if chunk_offset < remaining_buffer_view_region.byte_offset as usize {
                        let skip_bytes =
                            remaining_buffer_view_region.byte_offset as usize - chunk_offset;
                        let mut skip = Vec::new();
                        skip.resize(skip_bytes, 0);
                        println!("skip {}", skip.len());
                        reader.read_exact(&mut skip)?;
                        chunk_offset += skip_bytes;
                    }
                    let read_start = chunk_bytes.len();
                    let read_bytes = remaining_buffer_view_region.byte_length as usize;
                    chunk_bytes.resize(chunk_bytes.len() + read_bytes, 0);
                    let mut actual_read_bytes = 0;
                    loop {
                        // 実際に読めるサイズが必要とされるサイズと違うことがある？
                        let n = reader.read(&mut chunk_bytes[(read_start + actual_read_bytes)..(read_start + read_bytes)])?;
                        actual_read_bytes += n;
                        if actual_read_bytes == read_bytes {
                            break;
                        }
                        if n == 0 {
                            println!("read expected={} bytes, actual={} bytes", read_bytes, actual_read_bytes);
                            break
                        }
                    }
                    chunk_offset += read_bytes;
                }
                if chunk_offset < chunk_length as usize {
                    let mut skip_bytes = Vec::new();
                    skip_bytes.resize(chunk_length as usize - chunk_offset, 0);
                    println!("skip {}", skip_bytes.len());
                    reader.read_exact(&mut skip_bytes)?;
                    //println!("last chunk skipped: len={}", skip_bytes.len());
                }
                chunk_bytes.resize((chunk_bytes.len() + 3) / 4 * 4, 0);
                chunks.push(chunk_bytes);
            } else {
                let mut skip_bytes = Vec::new();
                skip_bytes.resize(chunk_length as usize, 0);
                println!("skip {}", skip_bytes.len());
                reader.read_exact(&mut skip_bytes)?;
                //println!("chunk skipped: {} len={}", chunk_index, chunk_length);
            }
            offset += 8 + chunk_length;
            chunk_index += 1;
        }
        Ok(chunks)
    }
}
