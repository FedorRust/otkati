import QtQuick
import QtTest
import Otkati 1.0

TestCase {
    id: testCase
    name: "Language"

    Component {
        id: faceComponent
        Face {}
    }

    function test_settings_start_in_english() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        compare(face.language, "en")
        compare(face.settingLabel("length"), "Length")
        compare(face.settingLabel("resolutionShare"), "Resolution share")
        compare(face.settingLabel("bitrate"), "Bitrate")
        compare(face.settingLabel("encoder"), "Encoder")
        compare(face.settingLabel("ringPlace"), "Ring place")
        compare(face.settingLabel("ringMonitor"), "Ring monitor")
        compare(face.settingLabel("bind"), "Bind")
        compare(face.settingLabel("language"), "Language")
        compare(face.settingLabel("theme"), "Theme")
        compare(face.settingLabel("otkatFolder"), "Otkat folder")
        compare(face.settingLabel("switch"), "Switch")
    }
}
