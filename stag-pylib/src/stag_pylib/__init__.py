from stag_pylib._core import Mesh


def main() -> None:
    mesh = Mesh(
        [0, 1, 2, 0, 3, 2],
        [
            0.0,
            0.0,
            0.0,
            1.0,
            1.0,
            1.0,
            2.0,
            2.0,
            2.0,
            -3.0,
            -3.0,
            -3.0,
        ],
    )
    mesh.export("test.obj")
    print("Exported a mesh!")


if __name__ == "__main__":
    print("Hello, World!")
    main()
