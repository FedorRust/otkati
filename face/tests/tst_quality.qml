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

    function test_changing_the_share_hands_it_and_leaves_the_bitrate() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.setBitrate(40)
        face.setResolutionShare(50)
        compare(face.resolutionShare, 50)
        compare(face.bitrate, 40)
        face.setResolutionShare(70)
        compare(face.resolutionShare, 70)
        compare(face.bitrate, 40)
    }

    function test_the_share_moves_from_50_to_100_in_steps_of_10() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        const steps = [50, 60, 70, 80, 90, 100]
        for (let i = 0; i < steps.length; i++) {
            face.setResolutionShare(steps[i])
            compare(face.resolutionShare, steps[i])
        }
        face.setResolutionShare(55)
        compare(face.resolutionShare, 100)
        face.setResolutionShare(40)
        compare(face.resolutionShare, 100)
        face.setResolutionShare(110)
        compare(face.resolutionShare, 100)
    }

    function test_the_bitrate_moves_from_8_to_80_one_mbit_at_a_time() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        face.setBitrate(8)
        compare(face.bitrate, 8)
        face.setBitrate(9)
        compare(face.bitrate, 9)
        face.setBitrate(25)
        compare(face.bitrate, 25)
        face.setBitrate(80)
        compare(face.bitrate, 80)
        face.setBitrate(7)
        compare(face.bitrate, 80)
        face.setBitrate(81)
        compare(face.bitrate, 80)
        face.setBitrate(8.5)
        compare(face.bitrate, 80)
        face.setResolutionShare(50)
        compare(face.bitrate, 80)
        compare(face.resolutionShare, 50)
    }
}
