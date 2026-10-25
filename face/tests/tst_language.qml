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

    function test_choosing_russian_in_settings_relabels_them() {
        const face = createTemporaryObject(faceComponent, null)
        verify(face)
        compare(face.languageChoice("en"), "English")
        compare(face.languageChoice("ru"), "Russian")

        face.chooseLanguage("ru")
        compare(face.language, "ru")
        compare(face.languageChoice("en"), "Английский")
        compare(face.languageChoice("ru"), "Русский")
        compare(face.settingLabel("length"), "Длина")
        compare(face.settingLabel("resolutionShare"), "Доля разрешения")
        compare(face.settingLabel("bitrate"), "Битрейт")
        compare(face.settingLabel("encoder"), "Кодировщик")
        compare(face.settingLabel("ringPlace"), "Место кольца")
        compare(face.settingLabel("ringMonitor"), "Монитор кольца")
        compare(face.settingLabel("bind"), "Бинд")
        compare(face.settingLabel("language"), "Язык")
        compare(face.settingLabel("theme"), "Тема")
        compare(face.settingLabel("otkatFolder"), "Папка откатов")
        compare(face.settingLabel("switch"), "Выключатель")
        face.showPlashka(Face.PlashkaEvent.NothingToSave)
        compare(face.plashkaText, "Сохранять нечего")

        face.chooseLanguage("en")
        compare(face.language, "en")
        compare(face.languageChoice("en"), "English")
        compare(face.languageChoice("ru"), "Russian")
        compare(face.settingLabel("length"), "Length")
        compare(face.settingLabel("otkatFolder"), "Otkat folder")
        compare(face.plashkaText, "Nothing to save")
    }
}
