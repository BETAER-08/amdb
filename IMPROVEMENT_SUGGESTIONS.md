# 코드베이스 개선 및 수정 항목

이 문서는 amdb 프로젝트의 코드를 분석한 결과 발견한 개선 및 수정이 필요한 항목들을 정리한 것입니다.

## 🔴 중요도 높음 (High Priority)

### 1. 빌드 실패 문제
**문제**: `ort-sys` 의존성 빌드 실패
- 현재 `fastembed` 라이브러리가 `ort-sys 2.0.0-rc.9`를 사용하는데, 이것이 `parcel.pyke.io`에서 바이너리를 다운로드하려다 DNS 해결 실패로 빌드가 안됨
- **영향**: 프로젝트를 로컬에서 빌드할 수 없음
- **위치**: `Cargo.toml` - fastembed 의존성
- **제안**: 
  - 오프라인 빌드 옵션 제공
  - 또는 다른 임베딩 라이브러리로 대체 고려
  - 빌드 문제 해결 가이드를 README에 추가

### 2. 테스트 코드 부재
**문제**: 프로젝트에 단위 테스트나 통합 테스트가 전혀 없음
- **영향**: 코드 품질 보증이 어렵고, 리팩토링 시 회귀 버그 발생 위험
- **제안**: 
  - 핵심 모듈에 단위 테스트 추가 (parser, indexer, vector_store)
  - CI/CD에서 자동으로 테스트를 실행하는 워크플로우 추가
  - 테스트 커버리지 측정 도구 도입

### 3. 에러 처리 개선 필요
**문제**: 여러 곳에서 에러를 무시하거나 간단히 출력만 함
- **위치들**:
  - `src/main.rs:44` - `eprintln!`로만 에러 출력
  - `src/daemon/watcher.rs:29` - `let _ = Indexer::scan_project(".");` 에러 무시
  - `src/core/indexer.rs:40-63` - 여러 곳에서 에러를 조용히 무시
- **제안**:
  - 적절한 에러 전파 및 로깅 구현
  - 사용자에게 의미 있는 에러 메시지 제공
  - 로깅 프레임워크 (tracing) 적극 활용

## 🟡 중요도 중간 (Medium Priority)

### 4. 하드코딩된 경로 및 설정값
**문제**: 여러 설정값이 코드에 하드코딩되어 있음
- **위치들**:
  - `.database/` 디렉토리 경로 하드코딩
  - `.amdb/` 디렉토리 경로 하드코딩
  - 무시할 디렉토리 목록 (`target`, `node_modules` 등)
  - Vector search limit 값 (10)이 하드코딩
- **제안**:
  - 설정 파일 지원 추가 (예: `.amdb/config.toml`)
  - 환경 변수로 경로 오버라이드 가능하게
  - CLI 옵션으로 주요 설정 조정 가능하게

### 5. README 문서 개선
**문제**: README에 불완전한 정보와 오류가 있음
- **위치**: `README.md`
  - 라인 22: "Option 2: Manual Download" 제목에 마크다운 포매팅 누락 (`#` 없음)
  - 라인 94: 불완전한 HTML 태그로 문서가 끝남
  - 실제 사용 예시와 출력 샘플 부족
- **제안**:
  - 마크다운 포매팅 수정
  - 더 자세한 사용 예시와 스크린샷 추가
  - 트러블슈팅 섹션 추가
  - 기여 가이드라인 추가

### 6. 성능 최적화 기회
**문제**: 잠재적인 성능 개선 포인트들
- **위치들**:
  - `src/core/indexer.rs:49-60` - 각 심볼마다 개별적으로 임베딩 생성 (배치 처리 가능)
  - `src/core/vector_store.rs:60-70` - 전체 벡터를 순회하며 검색 (인덱스 구조 고려)
  - `src/db/query.rs:30-38` - 트랜잭션마다 prepare statement 재생성
- **제안**:
  - 임베딩을 배치로 처리하여 성능 향상
  - 대규모 프로젝트를 위한 벡터 인덱싱 (HNSW, IVF 등) 고려
  - Prepared statement 재사용

### 7. .gitignore 중복
**문제**: `.gitignore` 파일에 중복된 항목
- **위치**: `/.gitignore` 라인 2와 8 - `.amdb/`가 두 번 나타남
- **제안**: 중복 제거

## 🟢 중요도 낮음 (Low Priority)

### 8. 코드 문서화 부족
**문제**: 공개 API와 복잡한 로직에 대한 문서화 주석 부족
- **제안**:
  - 주요 public 함수와 struct에 /// doc 주석 추가
  - 복잡한 알고리즘에 설명 주석 추가
  - `cargo doc`으로 문서 생성 가능하도록

### 9. CLI UX 개선
**문제**: CLI 사용성 개선 여지
- **제안**:
  - `--version` 플래그로 버전 표시
  - `--help`에 더 자세한 설명과 예시 추가
  - 진행 상황 표시 (progress bar) 추가
  - `init` 명령어에 초기화 확인 메시지

### 10. 보안 패턴 감지 기능 확장
**문제**: 현재 5개 패턴만 감지
- **위치**: `src/core/parser.rs:17-25`
- **제안**:
  - GitHub secret 패턴 추가
  - Database 연결 문자열 감지
  - Generic password/token 패턴 추가

### 11. Daemon 모드 개선
**문제**: Daemon 모드가 제한적
- **위치**: `src/daemon/watcher.rs`
- **제안**:
  - 모든 지원 언어 확장자 감지 (현재는 rs, py, js, ts만)
  - Debouncing 추가 (짧은 시간에 여러 변경 시 한 번만 재인덱싱)
  - 백그라운드 모드 지원

### 12. 벡터 스토어 파일명 일관성
**문제**: 벡터 스토어 관련 파일/디렉토리명이 혼재
- **위치**: 
  - `src/core/indexer.rs:24` - `vector` 디렉토리
  - `src/core/vector_store.rs:29` - `vectors.bin` 파일
- **제안**: 일관된 명명 규칙 사용 (예: `vectors/` 또는 `vector/`)

## 📋 추가 제안사항

### 13. CI/CD 파이프라인 강화
- 현재는 release 워크플로우만 있음
- **제안**:
  - PR에 대한 자동 테스트 워크플로우
  - Clippy 린트 체크
  - 보안 감사 (cargo audit)
  - 코드 포맷 체크 (rustfmt)

### 14. 크로스 플랫폼 테스트
- Windows에서의 동작 검증
- 경로 구분자 처리 확인 (PathBuf 사용 권장)

### 15. 사용자 피드백 수집
- 예시 프로젝트나 벤치마크 추가
- 사용자 경험 개선을 위한 피드백 채널
- Discussion 또는 Discord 커뮤니티

### 16. 보안 강화
- 생성된 context.md 파일이 민감한 정보를 포함할 수 있음
- .gitignore에 기본적으로 추가되어 있지만, 사용자에게 경고 메시지 표시 권장

### 17. 종속성 관리
- Cargo.toml에서 사용하지 않는 의존성 확인 (axum이 코드에서 사용되지 않는 것으로 보임)
- 의존성 버전 업데이트 정책 수립

---

**분석 일자**: 2026-01-30  
**분석 도구**: 자동 코드 분석  
**총 항목 수**: 17개 (High: 3, Medium: 4, Low: 5, 추가: 5)

## 다음 단계

이 문서의 내용을 GitHub Issue로 생성하려면:

```bash
# GitHub CLI를 사용하는 경우
gh issue create --title "코드베이스 개선 및 수정 항목" --body-file IMPROVEMENT_SUGGESTIONS.md --label "enhancement,documentation"

# 또는 GitHub 웹 인터페이스에서:
# 1. Issues 탭으로 이동
# 2. New Issue 클릭
# 3. 이 파일의 내용을 복사하여 붙여넣기
```
