TRUNCATE TABLE BILETE,
JOIN_PE,
PACHETE,
EVENIMENTE RESTART IDENTITY CASCADE;

INSERT INTO
    EVENIMENTE (ID_OWNER, nume, locatie, descriere, numarLocuri)
VALUES
    (
        3,
        'Clojure/conj',
        'Durham, NC',
        'Clojure programming conference exploring functional programming, immutability, and REPL-driven development',
        2
    ),
    (
        3,
        'CppCon Europe',
        'Berlin, Germany',
        'Premier C++ conference covering modern C++20/23 features, performance optimization, and best practices',
        5
    ),
    (
        3,
        'DevOps Enterprise Summit',
        'London, UK',
        'Conference on DevOps culture, CI/CD pipelines, infrastructure as code, and site reliability engineering',
        4
    ),
    (
        3,
        'Django Con',
        'Edinburgh, Scotland',
        'Python web framework conference featuring Django best practices, REST APIs, and deployment strategies',
        3
    ),
    (
        3,
        'dotNet Conf',
        'Seattle, WA',
        'Microsoft .NET conference covering C#, ASP.NET Core, Blazor, and MAUI cross-platform development',
        5
    ),
    (
        4,
        'Elixir Forum',
        'Krakow, Poland',
        'Conference on functional programming with Elixir and Phoenix framework for scalable applications',
        2
    ),
    (
        3,
        'GoLab Conference',
        'Florence, Italy',
        'International Go conference with workshops on microservices, concurrency patterns, and cloud-native development',
        3
    ),
    (
        4,
        'JavaScript World',
        'Amsterdam, Netherlands',
        'Full-stack JavaScript conference covering React, Node.js, TypeScript, and modern web development',
        4
    ),
    (
        4,
        'Kotlin/Everywhere',
        'Prague, Czech Republic',
        'Kotlin conference for Android and multiplatform development with hands-on workshops',
        1
    ),
    (
        3,
        'KubeCon + CloudNativeCon',
        'Paris, France',
        'Kubernetes and cloud-native technologies conference with workshops on container orchestration',
        5
    ),
    (
        4,
        'Machine Learning Conference',
        'Munich, Germany',
        'ML and AI conference featuring TensorFlow, PyTorch, transformers, and production ML systems',
        4
    ),
    (
        4,
        'PyData Summit',
        'London, UK',
        'Data science and machine learning conference focused on Python tools like NumPy, Pandas, and TensorFlow',
        3
    ),
    (
        4,
        'React Summit',
        'New York, NY',
        'React ecosystem conference covering hooks, server components, Next.js 14, and modern React patterns',
        5
    ),
    (
        3,
        'RustConf 2025',
        'San Francisco, CA',
        'Annual conference for Rust programming language enthusiasts featuring talks on systems programming, async runtime, and WebAssembly',
        0
    ),
    (
        3,
        'Scala Days',
        'Lausanne, Switzerland',
        'Functional programming conference focused on Scala language, Akka, and distributed systems',
        1
    ),
    (
        4,
        'Swift Summit',
        'Barcelona, Spain',
        'iOS and macOS development conference featuring SwiftUI, Combine, and app architecture patterns',
        2
    ),
    (
        4,
        'Vue.js Amsterdam',
        'Amsterdam, Netherlands',
        'Vue.js framework conference with talks on Composition API, Nuxt 3, and state management with Pinia',
        3
    ),
    (
        4,
        'WebAssembly Summit',
        'Tokyo, Japan',
        'Deep dive into WebAssembly, WASI, and running native code in the browser with Rust and C++',
        0
    );

INSERT INTO
    PACHETE (ID_OWNER, nume, locatie, descriere, numarLocuri)
VALUES
    (
        4,
        'AI & WebAssembly Future',
        'Tokyo & Munich',
        'Cutting-edge tech with WebAssembly Summit and Machine Learning Conference',
        0
    ),
    (
        3,
        'Backend Development Bundle',
        'European Circuit',
        'Master backend technologies with GoLab, Elixir Forum, and Django Con - microservices and distributed systems',
        3
    ),
    (
        3,
        'Cloud Native Complete',
        'Europe & Asia',
        'Master cloud technologies with KubeCon, WebAssembly Summit, and DevOps Enterprise Summit',
        5
    ),
    (
        4,
        'Data Science & ML Package',
        'UK & Germany Tour',
        'Comprehensive data science track combining PyData Summit and Machine Learning Conference',
        2
    ),
    (
        3,
        'Enterprise DevOps Track',
        'London & Paris',
        'DevOps Enterprise Summit with KubeCon for complete cloud-native DevOps mastery',
        4
    ),
    (
        4,
        'Full Stack JavaScript Pro',
        'Global JavaScript Tour',
        'JavaScript World, React Summit, and Vue.js Amsterdam for complete JS mastery',
        5
    ),
    (
        4,
        'Functional Programming Path',
        'Global Tour',
        'Explore functional paradigms with Elixir Forum, Scala Days, and Clojure/conj',
        1
    ),
    (
        3,
        'Microsoft Stack Package',
        'Seattle & Europe',
        'dotNet Conf with workshops on C#, Azure, and enterprise .NET development',
        3
    ),
    (
        4,
        'Mobile Development Suite',
        'Central Europe',
        'Cross-platform mobile development with Kotlin/Everywhere and Swift Summit',
        2
    ),
    (
        4,
        'Modern Web Stack Package',
        'Western Europe',
        'Full-stack modern web development with JavaScript World, Vue.js Amsterdam, and React Summit',
        4
    ),
    (
        4,
        'Python Developer Path',
        'UK & Scotland',
        'Python ecosystem deep dive with PyData Summit and Django Con',
        3
    ),
    (
        3,
        'React Ecosystem Bundle',
        'Amsterdam & New York',
        'Master React with Vue.js Amsterdam, React Summit, and Next.js workshops',
        5
    ),
    (
        3,
        'Systems Programming Track',
        'Multi-city Tour',
        'Complete systems programming experience with RustConf and CppCon - learn low-level optimization and memory safety',
        0
    ),
    (
        3,
        'Test Package - Min Seats',
        'Multi-city Test',
        'Test package with 3 events: Clojure/conj (2 seats), Django Con (3 seats), JavaScript World (4 seats)',
        999
    );

INSERT INTO
    JOIN_PE (PachetID, EvenimentID)
VALUES
    -- Systems Programming Track
    (1, 1),  -- RustConf
    (1, 2),  -- CppCon
    -- Backend Development Bundle
    (2, 3),  -- GoLab
    (2, 7),  -- Elixir Forum
    (2, 12), -- Django Con
    -- Modern Web Stack Package
    (3, 5),  -- JavaScript World
    (3, 13), -- Vue.js Amsterdam
    (3, 16), -- React Summit
    -- Data Science & ML Package
    (4, 4),  -- PyData Summit
    (4, 18), -- Machine Learning Conference
    -- Mobile Development Suite
    (5, 6),  -- Kotlin/Everywhere
    (5, 8),  -- Swift Summit
    -- Functional Programming Path
    (6, 7),  -- Elixir Forum
    (6, 14), -- Scala Days
    (6, 15), -- Clojure/conj
    -- Cloud Native Complete
    (7, 10), -- KubeCon
    (7, 9),  -- WebAssembly Summit
    (7, 17), -- DevOps Enterprise Summit
    -- Microsoft Stack Package
    (8, 11), -- dotNet Conf
    -- Python Developer Path
    (9, 4),  -- PyData Summit
    (9, 12), -- Django Con
    -- React Ecosystem Bundle
    (10, 13), -- Vue.js Amsterdam
    (10, 16), -- React Summit
    (10, 5),  -- JavaScript World
    -- Enterprise DevOps Track
    (11, 17), -- DevOps Enterprise Summit
    (11, 10), -- KubeCon
    -- AI & WebAssembly Future
    (12, 9),  -- WebAssembly Summit
    (12, 18), -- Machine Learning Conference
    -- Full Stack JavaScript Pro
    (13, 5),  -- JavaScript World
    (13, 16), -- React Summit
    (13, 13), -- Vue.js Amsterdam
    -- Test Package - Min Seats
    (14, 1),  -- Clojure/conj (2 seats)
    (14, 4),  -- Django Con (3 seats)
    (14, 8); -- JavaScript World (4 seats)
