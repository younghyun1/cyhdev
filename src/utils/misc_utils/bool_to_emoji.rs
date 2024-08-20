// bool 타입을 이모지로 변환해주는 간단한 함수이다. 클라이언트측에게 예쁘게 대답해주기 위해 사용.
// Very simple function to convert bool to the appropriate emoji. Reply neatly to the client using a message, as well as a bool!

pub fn bte(value: bool) -> &'static str {
    if value {
        "😊"
    } else {
        "😡"
    }
}
