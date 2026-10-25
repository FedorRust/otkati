import QtQuick
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "Quality"

    Component {
        id: faceComponent
        Face {}
    }

    function test_the_face_hands_the_whole_monitor_and_25_mbit() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        compare(face.resolutionShare, 100)
        compare(face.bitrate, 25)
    }
}
