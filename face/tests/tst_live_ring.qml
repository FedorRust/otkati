import QtQuick
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "LiveRing"

    Component {
        id: faceComponent
        Face {}
    }

    function test_the_face_has_no_microphone_only_knob() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        verify(face.knobNames.indexOf("ringPlace") !== -1)
        verify(face.knobNames.indexOf("encoder") !== -1)
        verify(face.knobNames.indexOf("bitrate") !== -1)
        verify(face.knobNames.indexOf("resolutionShare") !== -1)
        verify(face.knobNames.indexOf("microphoneOnly") === -1)
        verify(face.knobNames.indexOf("microphone") === -1)
    }
}
