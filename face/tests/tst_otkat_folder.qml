import QtQuick
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "OtkatFolder"

    Component {
        id: faceComponent
        Face {}
    }

    function test_face_hands_videos_otkaty_until_the_folder_changes() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        compare(face.otkatFolder, face.homePath + "/Videos/Откаты")
    }

    function test_changing_the_folder_hands_the_new_path_to_the_ring() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.setOtkatFolder("/mnt/other/Откаты")
        compare(face.otkatFolder, "/mnt/other/Откаты")
    }
}
