# IronPress Material Format

[IronPress](https://github.com/arocull/IronPress) is a texture management/compression tool designed to enforce texture consistency and naming conventions across large projects.

IronPress works off of a user-defined configuration file, which points to input and output directories for textures, and the corresponding channels and formats that should be compressed.
Usually you have to run IronPress manually via CLI, and then set up the materials in-engine yourself.

**StagToolkit can automatically import these materials for you** if you place the IronPress configuration directly into your project, so long as you install IronPress first ([see IronPress instructions](https://github.com/arocull/IronPress)).

## Example Usage

I want to combine and resize some baked textures from my 3D asset repository, and import them into Godot.

First, I output a default IronPress configuration into my Godot project: `$ ironpress --default cactus.ironpress`.
I then modify the configuration with any text editor to look like the following.

```json
{
    "input": "/threed/projects/abyss/props/cacti/textures/", // original textures
    "output": "./textures/", // compressed textures output into this relative directory
    "flip_normals": false,
    "materials": {
        "mat_cactus": { // Name of material
            "max_dimension": 1024, // scale textures down to this max dimension
            "alpha": false, // basecolor map does not use transparency
            "channels": [
                "basecolor",
                "arm", // combine the ambient occlusion + roughness + metallic maps
                "normal"
                // other channel options available as needed, see IronPress repository
            ]
        } // ...multiple materials can be added to one configuration
    }
}
```

![](images/ironpress-1.png)

You can re-import the IronPress configuration when ready.

![](images/ironpress-2.png)

The importer will run IronPress through terminal, optimize and re-import the textures, and output materials for all configured materials.

![](images/ironpress-3.png)

Because everything has an edge-case, **re-importing the `.ironpress` configuration should only modify what is necessary**, leaving extra customizations in-tact.

Resources are output in text format for easy version-control management.
