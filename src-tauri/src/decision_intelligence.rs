use serde_json::{json, Value};

// CEO Decision Intelligence: cognitive assets beyond mental models.
// These are seeded as first-class records so the Decision Engine can reference
// them without duplicating data in the frontend.

pub const FRAMEWORK_SEEDS: &[(&str, &str, &str, &str, &str, &str)] = &[
    (
        "strategy-kernel",
        "战略内核",
        "回答“我们到底应该解决什么问题”。",
        "目标 + 约束 + 优势 + 关键矛盾",
        "战略方向、新业务、重大转型、年度规划",
        "目标\n约束\n优势\n关键矛盾",
    ),
    (
        "five-forces",
        "波特五力",
        "分析行业结构对利润与竞争的影响。",
        "供应商、客户、潜在进入者、替代品、现有竞争者",
        "进入新市场、行业分析、竞争判断、定价",
        "五力强度\n行业利润\n结构性威胁",
    ),
    (
        "value-chain",
        "价值链",
        "拆解价值如何被创造、传递和获取。",
        "价值创造、价值传递、价值获取",
        "成本优化、利润结构、商业模式、差异化",
        "价值环节\n成本结构\n利润来源",
    ),
    (
        "jtbd",
        "JTBD 待完成任务",
        "理解用户真正要完成的任务，而不是只分析产品。",
        "用户任务、场景、痛点、期望结果",
        "产品定义、选品、需求验证、内容方向",
        "用户任务\n场景\n痛点\n期望结果",
    ),
    (
        "strategic-inflection",
        "战略拐点",
        "发现行业结构发生重大变化的信号。",
        "技术、用户、平台、成本、竞争、监管",
        "行业变化、技术冲击、平台迁移、竞争突变",
        "变化维度\n拐点判断\n应对选项",
    ),
    (
        "innovators-dilemma",
        "创新者窘境",
        "判断成熟业务是否被新技术或新模式冲击。",
        "成熟业务惯性、颠覆式创新、低端市场",
        "成熟业务防御、新技术评估、自我颠覆",
        "被颠覆风险\n防御策略\n自我革新",
    ),
    (
        "reversible-irreversible",
        "可逆 / 不可逆决策",
        "根据可逆性决定分析深度、决策速度与验证方式。",
        "可逆性判断、分析深度、验证强度",
        "所有重要决策的前置判断",
        "可逆性等级\n分析深度\n验证方式",
    ),
    (
        "premortem",
        "预演失败",
        "假设项目已经失败，倒推最可能的失败原因。",
        "失败假设、原因倒推、预防措施",
        "项目启动、大额投入、高风险行动",
        "失败原因\n致命风险\n预防实验",
    ),
];

pub const PRINCIPLE_SEEDS: &[(&str, &str, &str, &str, &str, &str, &str)] = &[
    (
        "long-termism",
        "长期主义",
        "把关键资源押注在能够随时间复利的方向上。",
        "当短期收益与长期价值冲突时，优先保护长期复利资产。",
        "战略投入、研发、品牌、客户关系",
        "为了季度目标牺牲客户与团队信任。",
        "Amazon 长期亏损投入基础设施换取长期护城河。",
    ),
    (
        "customer-obsession",
        "客户至上",
        "从客户真实问题出发，而不是从内部指标出发。",
        "先确认客户是否真的因此变得更好，再判断是否值得做。",
        "产品、服务、内容、定价、售后",
        "把“客户至上”变成口号，实际只优化内部 KPI。",
        "Amazon 从客户痛点反推产品，而不是先做产品再找客户。",
    ),
    (
        "day-1",
        "Day 1",
        "保持第一天的心态，警惕组织进入衰退惯性。",
        "当流程比结果重要、官僚比速度重要时，重新回到 Day 1。",
        "组织活力、创新、危机意识、执行力",
        "用制度和流程掩盖对结果的回避。",
        "Amazon 的 Day 1 文化：永远把今天当作创业第一天。",
    ),
    (
        "effective-executive",
        "有效管理者",
        "把稀缺的注意力投入真正改变结果的事情。",
        "每次做决策前问：这值得 CEO 亲自决定吗？",
        "优先级、授权、会议、时间配置",
        "什么都想管，最终重要的事没有推进。",
        "Peter Drucker：先记录时间，再消除无效工作。",
    ),
    (
        "strengths-management",
        "优势管理",
        "围绕优势设计角色，而不是只修补弱点。",
        "把每个人放到能发挥最大优势的位置。",
        "招聘、分工、组织设计、核心团队",
        "平均主义地修补所有人的所有弱点。",
        "把擅长洞察的人从执行琐事中解放出来。",
    ),
    (
        "principle-based-management",
        "原则化管理",
        "用清晰原则代替临时命令，保证判断一致性。",
        "遇到原则没有覆盖的新情况，更新原则而不是绕过原则。",
        "授权、规模化、冲突处理、组织治理",
        "原则只写不执行，或领导带头破坏原则。",
        "Ray Dalio 把原则写成系统，用于可重复的高质量决策。",
    ),
    (
        "ceo-attention",
        "CEO 注意力配置",
        "CEO 最稀缺的资源是注意力，必须显式配置。",
        "把 80% 注意力放在不可逆、高影响、只有 CEO 能做的事。",
        "战略、关键人才、大额资源、外部关系",
        "被会议和日常事务占据，没有时间思考方向。",
        "把 CEO 的一周拆成战略块、决策块和执行块。",
    ),
];

pub const LENS_SEEDS: &[(&str, &str, &str, &str, &str, &str)] = &[
    (
        "strategic",
        "战略决策",
        "市场、竞争、能力、资源、风险、长期价值",
        "first-principles,second-order-thinking,opportunity-cost,circle-of-competence,margin-of-safety",
        "strategy-kernel,five-forces,value-chain",
        "决定方向与长期资源配置。",
    ),
    (
        "investment",
        "投资决策",
        "收益、概率、下行风险、机会成本、安全边际",
        "probabilistic-thinking,base-rate,inversion,opportunity-cost,margin-of-safety",
        "strategy-kernel,five-forces",
        "判断是否投入资金与资源。",
    ),
    (
        "product",
        "产品决策",
        "用户需求、价值、验证、反馈、资源",
        "first-principles,jtbd,feedback-loop,opportunity-cost,reversible-irreversible",
        "jtbd,value-chain",
        "判断做什么产品与如何验证。",
    ),
    (
        "market-entry",
        "市场进入",
        "市场规模、竞争、用户、自身能力、进入成本",
        "five-forces,jtbd,circle-of-competence,opportunity-cost,probabilistic-thinking",
        "five-forces,jtbd,strategy-kernel",
        "判断是否进入新市场。",
    ),
    (
        "organization",
        "组织决策",
        "人、激励、能力、组织结构、长期影响",
        "incentives,circle-of-competence,second-order-thinking,opportunity-cost",
        "strategy-kernel",
        "判断组织如何设计与调整。",
    ),
    (
        "talent",
        "人才决策",
        "能力、潜力、匹配、激励、机会成本",
        "strengths-management,circle-of-competence,incentives,opportunity-cost",
        "strategy-kernel",
        "判断关键人才的引进、保留与使用。",
    ),
    (
        "finance",
        "财务决策",
        "现金流、收益、风险、安全边际、机会成本",
        "margin-of-safety,probabilistic-thinking,base-rate,opportunity-cost",
        "value-chain",
        "判断资金如何配置与保护。",
    ),
    (
        "crisis",
        "危机决策",
        "生存、速度、损失控制、不可逆后果",
        "inversion,margin-of-safety,second-order-thinking,probabilistic-thinking",
        "premortem",
        "在威胁生存时快速止损与保护关键资源。",
    ),
];

fn now_value(applied_at: &str) -> Value {
    json!({ "createdAt": applied_at, "updatedAt": applied_at })
}

pub fn migrate(connection: &rusqlite::Connection, applied_at: &str) -> Result<(), String> {
    let already: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version=12)",
            [],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if already {
        return Ok(());
    }

    for (slug, name, purpose, components, use_cases, output) in FRAMEWORK_SEEDS {
        let id = format!("decision-framework-{slug}");
        let data = json!({
            "id": id, "entity": "decisionFrameworks", "slug": slug, "name": name,
            "purpose": purpose, "components": components, "useCases": use_cases, "output": output,
            "status": "active", "createdAt": applied_at, "updatedAt": applied_at
        });
        let raw = serde_json::to_string(&data).map_err(|e| e.to_string())?;
        connection
            .execute(
                "INSERT OR IGNORE INTO records(id,entity,data_json,title,body,created_at,updated_at) VALUES(?1,'decisionFrameworks',?2,?3,?4,?5,?5)",
                rusqlite::params![id, raw, name, raw, applied_at],
            )
            .map_err(|e| e.to_string())?;
    }

    for (slug, name, definition, decision_rule, use_cases, anti_patterns, example) in
        PRINCIPLE_SEEDS
    {
        let id = format!("ceo-principle-{slug}");
        let data = json!({
            "id": id, "entity": "ceoPrinciples", "slug": slug, "name": name,
            "definition": definition, "decisionRule": decision_rule, "useCases": use_cases,
            "antiPatterns": anti_patterns, "example": example,
            "status": "active", "createdAt": applied_at, "updatedAt": applied_at
        });
        let raw = serde_json::to_string(&data).map_err(|e| e.to_string())?;
        connection
            .execute(
                "INSERT OR IGNORE INTO records(id,entity,data_json,title,body,created_at,updated_at) VALUES(?1,'ceoPrinciples',?2,?3,?4,?5,?5)",
                rusqlite::params![id, raw, name, raw, applied_at],
            )
            .map_err(|e| e.to_string())?;
    }

    for (slug, name, focus, model_slugs, framework_slugs, description) in LENS_SEEDS {
        let id = format!("decision-lens-{slug}");
        let data = json!({
            "id": id, "entity": "decisionLenses", "slug": slug, "name": name,
            "focus": focus, "recommendedModelSlugs": model_slugs, "recommendedFrameworkSlugs": framework_slugs,
            "description": description, "status": "active", "createdAt": applied_at, "updatedAt": applied_at
        });
        let raw = serde_json::to_string(&data).map_err(|e| e.to_string())?;
        connection
            .execute(
                "INSERT OR IGNORE INTO records(id,entity,data_json,title,body,created_at,updated_at) VALUES(?1,'decisionLenses',?2,?3,?4,?5,?5)",
                rusqlite::params![id, raw, name, raw, applied_at],
            )
            .map_err(|e| e.to_string())?;
    }

    let _ = now_value(applied_at);
    connection
        .execute(
            "INSERT OR IGNORE INTO schema_migrations(version,applied_at) VALUES(12,?1)",
            rusqlite::params![applied_at],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_v12_seeds_cognitive_assets() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);
                 CREATE TABLE records(id TEXT PRIMARY KEY, entity TEXT NOT NULL, data_json TEXT NOT NULL, title TEXT NOT NULL DEFAULT '', body TEXT NOT NULL DEFAULT '', created_at TEXT NOT NULL, updated_at TEXT NOT NULL, archived_at TEXT, deleted_at TEXT);",
            )
            .unwrap();
        migrate(&connection, "1000").unwrap();

        let count = |entity: &str| -> i64 {
            connection
                .query_row(
                    "SELECT COUNT(*) FROM records WHERE entity=?1",
                    rusqlite::params![entity],
                    |row| row.get(0),
                )
                .unwrap()
        };
        assert_eq!(count("decisionFrameworks"), 8);
        assert_eq!(count("ceoPrinciples"), 7);
        assert_eq!(count("decisionLenses"), 8);

        // idempotent: running again must not duplicate
        migrate(&connection, "1001").unwrap();
        assert_eq!(count("decisionFrameworks"), 8);
        assert_eq!(count("ceoPrinciples"), 7);
        assert_eq!(count("decisionLenses"), 8);
    }
}
